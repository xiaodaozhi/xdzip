"""Exercise the Qt-synchronized HTML contract in the caller's actual zoom environment."""
from playwright.sync_api import expect
from io import BytesIO
import base64
from PIL import Image


def verify(page, ratio, check, tag, output=None):
    keyboard = page.page.keyboard if hasattr(page, 'page') else page.keyboard
    page.evaluate('window.syncSavedState = {...state, selected:new Set(state.selected), expanded:new Set(state.expanded)}')

    def box(selector):
        return page.locator(selector).bounding_box()

    def center(rect, axis):
        return rect[axis] + rect['width' if axis == 'x' else 'height'] / 2

    for selector in ['#new-tab', '#title-settings', '#list-view', '#grid-view', '#sort-direction', '#hide-inspector']:
        button, glyph = box(selector), box(selector + ' svg')
        for axis in ['x', 'y']:
            check(tag + '/icon center ' + selector + axis, center(glyph, axis), center(button, axis), .5)
    title = box('.titlebar')
    for selector in ['.traffic', '#new-tab', '#title-settings']:
        check(tag + '/title center ' + selector, center(box(selector), 'y'), center(title, 'y'), .5)
    for selector in ['#open', '#create', '#test-archive']:
        glyph, label = box(selector + ' svg'), box(selector + ' .action-label')
        check(tag + '/toolbar gap ' + selector, label['x'] - glyph['x'] - glyph['width'], 6, .06)
    fixed = [box(s) for s in ['#list-view', '#grid-view', '#sort-direction']]
    for i, rect in enumerate(fixed):
        check(tag + '/view width ' + str(i), rect['width'], 28, .06)
        check(tag + '/view height ' + str(i), rect['height'], 28, .06)
        if i:
            check(tag + '/view gap ' + str(i), rect['x'] - fixed[i-1]['x'] - fixed[i-1]['width'], 6.4, .06)
    for selector in ['#hide-inspector', '#list-view', '#grid-view', '#sort-direction']:
        radius = page.locator(selector).evaluate('(n)=>parseFloat(getComputedStyle(n).borderRadius)') * ratio
        check(tag + '/rounded control ' + selector, radius > 0, True, 0)
    check(tag + '/primary blue', page.locator('#extract').evaluate('(n)=>getComputedStyle(n).backgroundColor'), 'rgb(25, 86, 255)', 0)
    check(tag + '/root label', page.locator('.tree-root .tree-label').inner_text(), '根目录', 0)
    check(tag + '/archive caption', page.locator('.archive-caption').inner_text(), '档案文件', 0)
    check(tag + '/caption center', center(box('.archive-caption'), 'y'), center(box('#breadcrumbs'), 'y'), .5)
    check(tag + '/root direct count', page.locator('#entry-count').inner_text(), '9 项', 0)
    page.locator('#file-rows [data-id=guide]').hover()
    colors = page.locator('#file-rows [data-id=guide] td').evaluate_all('(nodes)=>nodes.map(n=>getComputedStyle(n).backgroundImage)')
    check(tag + '/whole row hover', len(set(colors)), 1, 0)
    check(tag + '/row hover background layer', colors[0], 'linear-gradient(rgb(237, 242, 255), rgb(237, 242, 255))', 0)
    # The real pointer was verified above. Headed Chromium clears its synthetic pointer
    # while capturing; freeze that already-observed CSS state for the pixel measurement.
    surface = page.page if hasattr(page, 'page') else page
    cdp = surface.context.new_cdp_session(surface)
    cdp.send('DOM.enable')
    cdp.send('CSS.enable')
    document = cdp.send('DOM.getDocument', {'depth': -1, 'pierce': True})['root']
    def row_node(node):
        attrs = node.get('attributes', [])
        attributes = dict(zip(attrs[::2], attrs[1::2]))
        if node.get('nodeName') == 'TR' and attributes.get('data-id') == 'guide':
            return node['nodeId']
        children = node.get('children', []) + ([node['contentDocument']] if 'contentDocument' in node else [])
        return next((result for child in children if (result := row_node(child))), None)
    node = row_node(document)
    assert node, 'Hover row is missing from the browser DOM'
    cdp.send('CSS.forcePseudoState', {'nodeId': node, 'forcedPseudoClasses': ['hover']})
    try:
        cell = page.locator('#file-rows [data-id=guide] td').first
        # Browser-native page zoom maps CSS coordinates to capture DIPs. DPR only
        # affects image sampling, never the UI's logical component dimensions.
        rect = cell.bounding_box()
        zoom = cdp.send('Page.getLayoutMetrics')['cssVisualViewport']['zoom']
        clip = {key: rect[key] * zoom for key in ['x', 'y', 'width', 'height']}
        capture = cdp.send('Page.captureScreenshot', {'format': 'png',
                           'clip': {**clip, 'scale': 1}, 'captureBeyondViewport': True})
        image = Image.open(BytesIO(base64.b64decode(capture['data']))).convert('RGB')
        if output:
            image.save(str(output / 'row-hover-cell.png'))
        check(tag + '/row hover painted', list(image.getpixel((image.width // 4, image.height // 2))), [237, 242, 255], 0)
        geometry = cell.evaluate(r"""n=>{
            const band=getComputedStyle(n), line=getComputedStyle(n,'::after');
            const removed=band.backgroundSize.match(/-\s*([\d.]+)px/);
            return {top:parseFloat(band.backgroundPositionY),removed:removed?Number(removed[1]):0,line:parseFloat(line.height)};
        }""")
    finally:
        cdp.send('CSS.forcePseudoState', {'nodeId': node, 'forcedPseudoClasses': []})
        cdp.detach()
    check(tag + '/row band upper inset', geometry['top'] * ratio, 1, .06)
    check(tag + '/row separator height', geometry['line'] * ratio, 1, .06)
    check(tag + '/row separator equal gaps', (geometry['removed'] - geometry['top'] - geometry['line']) * ratio, geometry['top'] * ratio, .06)
    for directory in ['photos', 'ice']:
        if page.locator('[data-directory=' + directory + ']').get_attribute('aria-expanded') != 'true':
            page.locator('[data-toggle=' + directory + ']').click()
    guide, disclosure = box('[data-directory=ice] .tree-guide-parent'), box('[data-directory=photos] .tree-disclosure')
    check(tag + '/guide arrow center', center(guide, 'x'), center(disclosure, 'x'), .01)
    page.locator('[data-directory=ice] .tree-label').hover()
    page.page.wait_for_timeout(500)
    expect(page.locator('.ui-tooltip')).to_have_count(0)
    page.locator('[data-directory=shore]').click()
    crumb = page.locator('[data-breadcrumb=ice]')
    crumb.hover()
    head, item = box('.location'), crumb.bounding_box()
    check(tag + '/breadcrumb height', item['height'], 28, .06)
    check(tag + '/breadcrumb center', center(item, 'y'), center(head, 'y'), .5)
    for i, selector in enumerate(['#list-view', '#grid-view', '#sort-direction']):
        check(tag + '/fixed tool anchor ' + selector, box(selector)['x'], fixed[i]['x'], .06)
    page.locator('[data-directory=""]').click()
    page.locator('#grid-view').click()
    expect(page.locator('thead')).not_to_be_visible()
    card = page.locator('.grid-entry[data-id=photos]')
    checkbox = card.locator('input')
    page.locator('#open').hover()
    expect(checkbox).not_to_be_visible()
    card.hover()
    expect(checkbox).to_be_visible()
    check(tag + '/grid left inset', card.bounding_box()['x'] - box('.table-scroll')['x'], 16, .06)
    for axis in ['x', 'y']:
        check(tag + '/grid checkbox inset ' + axis, checkbox.bounding_box()[axis] - card.bounding_box()[axis], 8, .8)
    checkbox.click()
    page.locator('#open').hover()
    expect(checkbox).to_be_checked()
    expect(checkbox).to_be_visible()
    if output:
        page.locator('.window').screenshot(path=str(output / 'grid-selected.png'))
    checkbox.click()
    page.locator('#hide-inspector').click()
    expect(page.locator('#properties-toggle')).to_be_visible()
    check(tag + '/inspector toggle width', box('#properties-toggle')['width'], 28, .06)
    check(tag + '/inspector toggle height', box('#properties-toggle')['height'], 28, .06)
    page.locator('#properties-toggle').click()
    if output:
        page.locator('.window').screenshot(path=str(output / 'grid.png'))
    page.locator('#list-view').click()
    expect(page.locator('thead')).to_be_visible()
    for key in ['type', 'modified']:
        button = page.locator('[data-sort=' + key + ']')
        button.click()
        expect(button.locator('..')).to_have_attribute('aria-sort', 'ascending')
        button.click()
        expect(button.locator('..')).to_have_attribute('aria-sort', 'descending')
    page.locator('#sort-name').click()
    page.locator('#more-actions').click()
    menu, button = box('.ui-menu'), box('#more-actions')
    check(tag + '/more popup gap', menu['y'] - button['y'] - button['height'], 4, .06)
    shadow = page.locator('.ui-menu').evaluate('(n)=>getComputedStyle(n).boxShadow')
    check(tag + '/soft shadow present', shadow != 'none' and 'px' in shadow, True, 0)
    keyboard.press('Escape')
    original_theme = page.evaluate('state.theme')
    page.evaluate('setTheme("dark")')
    divider = page.locator('#extract-menu-button').evaluate('(n)=>getComputedStyle(n).borderLeftColor')
    check(tag + '/dark split divider', divider, 'rgb(54, 61, 75)', 0)
    page.evaluate('(theme)=>setTheme(theme)', original_theme)
    for selector, placement in [('#sort-direction', 'bottom'), ('#tasks', 'top')]:
        page.locator(selector).click()
        expect(page.locator('.ui-menu')).to_have_count(1)
        menu, button = box('.ui-menu'), box(selector)
        gap = menu['y'] - button['y'] - button['height'] if placement == 'bottom' else button['y'] - menu['y'] - menu['height']
        check(tag + '/popup gap ' + selector, gap, 4, .06)
        check(tag + '/popup right ' + selector, menu['x'] + menu['width'], button['x'] + button['width'], .06)
        if output:
            page.locator('.window').screenshot(path=str(output / (placement + '-menu.png')))
        keyboard.press('Escape')
        expect(page.locator(selector)).not_to_be_focused()
    page.locator('[data-directory=""]').click(button='right')
    expect(page.locator('[data-command=directory-search]')).to_be_visible()
    check(tag + '/directory separator', page.locator('[data-command=directory-search]').evaluate('(n)=>n.previousElementSibling.getAttribute("role")'), 'separator', 0)
    keyboard.press('Escape')
    expect(page.locator('.tree-count')).to_have_count(0)
    for selector in ['.tree-root', '[data-directory=shore]']:
        style = page.locator(selector).evaluate('(n)=>{const s=getComputedStyle(n);return [parseFloat(s.borderTopLeftRadius),parseFloat(s.borderBottomLeftRadius),s.outlineStyle]}')
        check(tag + '/square tree left ' + selector, style[:2], [0, 0], 0)
        check(tag + '/no tree outline ' + selector, style[2], 'none', 0)
    line = page.locator('.tree-root').evaluate('(n)=>{const s=getComputedStyle(n,"::before");return [s.top,s.bottom]}')
    check(tag + '/full-height tree indicator', line, ['0px', '0px'], 0)
    nested = page.evaluate('''() => { const entry = files.find(n=>n.parent); return {name:entry.name,path:archivePath(entry)}; }''')
    page.evaluate('(query)=>{state.query=query;state.path="";render()}', nested['name'])
    expect(page.locator('#file-rows .entry-name')).to_contain_text([nested['path']])
    page.evaluate('state.query=""; render()')
    expect(page.locator('#file-rows tr')).to_have_count(9)
    # Inject owned in-memory boundary samples; do not read archives or external data.
    page.evaluate('''() => {
        window.syncSavedExpanded = new Set(state.expanded);
        for(let i=0;i<14;i++) {
            const n={id:`sync-depth-${i}`, parent:i?`sync-depth-${i-1}`:'photos', name:i?'nested':'GitHub Desktop.app', folder:true};
            folders.push(n); byId.set(n.id,n); state.expanded.add(n.id);
        }
        renderTree();
    }''')
    widths = page.locator('.tree-scroll').evaluate('n=>({scroll:n.scrollWidth,viewport:n.clientWidth})')
    check(tag + '/expanded tree scroll', widths['scroll'] > widths['viewport'], True, 0)
    extent = page.locator('.tree-scroll').evaluate('''n => {
        const hierarchy=n.querySelector('.tree-hierarchy');
        const start=hierarchy.getBoundingClientRect().left;
        const end=Math.max(...[...hierarchy.querySelectorAll('.tree-label')].map(label=>{
            const range=document.createRange();range.selectNodeContents(label);return range.getBoundingClientRect().right;
        }));
        return {actual:n.scrollWidth, expected:Math.max(n.clientWidth,end-start+parseFloat(getComputedStyle(hierarchy.querySelector('.tree-row')).paddingRight))};
    }''')
    check(tag + '/expanded tree no trailing void', extent['actual'] * ratio, extent['expected'] * ratio, 1)

    page.evaluate('''() => {
        state.expanded.delete('photos'); renderTree();
    }''')
    widths = page.locator('.tree-scroll').evaluate('n=>({scroll:n.scrollWidth,viewport:n.clientWidth})')
    check(tag + '/collapsed tree no phantom width', widths['scroll'] <= widths['viewport'] + 1, True, 0)
    page.evaluate('''() => {
        folders=folders.filter(n=>!n.id.startsWith('sync-depth-'));
        for(let i=0;i<14;i++) byId.delete(`sync-depth-${i}`);
        state.expanded=window.syncSavedExpanded; delete window.syncSavedExpanded;
        for(let i=0;i<270;i++) {const n={id:`sync-file-${i}`,parent:'',name:`entry-${i}.txt`,size:i,compressed:i,type:'文本文件',modified:'2026/10/13'};files.push(n);entries.push(n);byId.set(n.id,n);}
        render();
    }''')
    expect(page.locator('#file-rows tr')).to_have_count(279)
    page.evaluate('''() => {
        for(let i=0;i<270;i++) byId.delete(`sync-file-${i}`);
        files.splice(files.length-270,270); entries.splice(entries.length-270,270); render();
    }''')
    expect(page.locator('#file-rows tr')).to_have_count(9)
    page.evaluate('Object.assign(state, window.syncSavedState); delete window.syncSavedState; render()')
