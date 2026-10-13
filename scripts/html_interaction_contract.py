"""Exercise the Qt-synchronized HTML contract in the caller's actual zoom environment."""
from playwright.sync_api import expect


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
    for directory in ['photos', 'ice']:
        if page.locator('[data-directory=' + directory + ']').get_attribute('aria-expanded') != 'true':
            page.locator('[data-toggle=' + directory + ']').click()
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
        expect(page.locator(selector)).to_be_focused()
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
