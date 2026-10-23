#!/usr/bin/env python3
"""Drive main-window and shared-control interactions in the offline HTML delivery.

These are design-demo checks, not archive codec or native-platform evidence.
Commands are exercised through pointer/keyboard input; state reads only assert
outcomes. The suite never supplies files or allows a network request.
"""
import json
import os
from pathlib import Path
from playwright.sync_api import sync_playwright, expect

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = Path(os.environ.get('XDZIP_HTML_CONTROLS_OUTPUT', ROOT / 'dist/html-controls-review'))
OUTPUT.mkdir(parents=True, exist_ok=True)

MENU_DIAGNOSTICS = r"""
(() => {
    const timeline = [], limit = 200;
    const node = value => value instanceof Element ? {
        tag:value.tagName.toLowerCase(), id:value.id, role:value.getAttribute('role'),
        class_name:value.getAttribute('class'),
        directory:value.getAttribute('data-directory'), command:value.getAttribute('data-command'),
        connected:value.isConnected,
    } : {tag:value === document ? '#document' : value === window ? '#window' : '#other'};
    const rect = value => {
        const box = value.getBoundingClientRect();
        return {x:box.x,y:box.y,width:box.width,height:box.height};
    };
    const scroll = value => ({node:node(value),x:value.scrollLeft,y:value.scrollTop,
        width:value.clientWidth,height:value.clientHeight,scrollWidth:value.scrollWidth,scrollHeight:value.scrollHeight});
    const append = (type, detail) => {
        timeline.push({at_ms:performance.now(),type,...detail});
        if (timeline.length > limit) timeline.shift();
    };
    const snapshot = target => {
        const ancestors = [];
        for (let current = target; current instanceof Element; current = current.parentElement)
            ancestors.push(scroll(current));
        const root = document.scrollingElement;
        if (root && !ancestors.some(item => item.node.tag === root.tagName.toLowerCase()))
            ancestors.push(scroll(root));
        let stateSnapshot;
        try { stateSnapshot = {path:state.path,selected:[...state.selected],expanded:[...state.expanded]}; }
        catch { stateSnapshot = {available:false}; }
        return {active_element:node(document.activeElement),document_has_focus:document.hasFocus(),
            target:target ? {...node(target),rect:rect(target),
                contextmenu_handler_registered:typeof target.oncontextmenu === 'function',
                keydown_handler_registered:typeof target.onkeydown === 'function'} : null,
            viewport:{width:innerWidth,height:innerHeight,dpr:devicePixelRatio},
            fonts_status:document.fonts.status,scroll_positions:ancestors,state:stateSnapshot,
            menus:[...document.querySelectorAll('.ui-menu')].slice(0,8).map(menu =>
                ({node:node(menu),rect:rect(menu),html:menu.outerHTML.slice(0,12000)})),
            timeline:[...timeline]};
    };
    window.__xdzipMenuDiagnostics = {snapshot};
    for (const capture of [true,false]) {
        for (const type of ['contextmenu','keydown','keyup']) document.addEventListener(type,event => {
            if (type !== 'contextmenu' && !['ContextMenu','F10','Escape'].includes(event.key)) return;
            append(type,{phase:capture?'capture':'bubble',target:node(event.target),key:event.key,
                shift:event.shiftKey,default_prevented:event.defaultPrevented,
                active_element:node(document.activeElement)});
        },capture);
    }
    document.addEventListener('scroll',event => {
        const target = event.target === document ? document.scrollingElement : event.target;
        append('scroll',{target:node(event.target),trusted:event.isTrusted,position:target ? scroll(target) : null});
    },true);
    window.addEventListener('resize',() => append('resize',{width:innerWidth,height:innerHeight}));
    new MutationObserver(records => {
        for (const record of records) for (const [action,nodes] of
            [['menu_added',record.addedNodes],['menu_removed',record.removedNodes]]) {
            for (const item of nodes) if (item instanceof Element) {
                const menus = item.matches('.ui-menu') ? [item] : [...item.querySelectorAll('.ui-menu')];
                for (const menu of menus) append(action,{menu:node(menu),rect:rect(menu)});
            }
        }
    }).observe(document,{childList:true,subtree:true});
})();
"""


def main():
    checks, errors, requests, directory_cases, queued_scroll_cases = [], [], [], [], []
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(executable_path=os.environ.get('XDZIP_CHROMIUM', '/usr/bin/chromium'),
                                             args=['--no-sandbox', '--disable-dev-shm-usage'])
        context = browser.new_context(viewport={'width': 1588, 'height': 1000}, locale='zh-CN', has_touch=True)
        context.set_default_timeout(10000)
        context.route('**/*', lambda route: (requests.append(route.request.url), route.abort()))
        holder = context.new_page()
        holder.set_content((ROOT / 'dist/xdzip-hifi-design.html').read_text())
        canvas = holder.evaluate('window.XDZIP_CANVAS')
        page = context.new_page()
        page.on('pageerror', lambda error: errors.append(str(error)))
        # This test-only observer is installed before the design scripts. It reads
        # events and DOM changes without preventing, dispatching or mutating them.
        assert '<head>' in canvas
        page.set_content(canvas.replace('<head>', '<head><script>' + MENU_DIAGNOSTICS + '</script>', 1))
        page.evaluate('document.fonts.ready')
        expect(page.locator('#file-rows tr')).to_have_count(9)
        expect(page.locator('[data-tab]')).to_have_count(2)
        expect(page.locator('#tree [role=treeitem]')).to_have_count(4)
        checks.append('Initial archive, table and directory controls load offline')
        tree_snapshot = page.evaluate('({path:state.path,selected:[...state.selected],expanded:[...state.expanded]})')

        directory_attempt = 0

        def open_directory_menu(row, method, *, language, text_scale, scenario='directory_matrix'):
            nonlocal directory_attempt
            directory_attempt += 1
            context = {'attempt':directory_attempt,'language':language,'text_scale':text_scale,
                       'method':method,'row_id':row.get_attribute('data-directory'),'scenario':scenario}
            before = row.evaluate('(row)=>__xdzipMenuDiagnostics.snapshot(row)')
            try:
                if method == 'mouse':
                    row.click(button='right')
                else:
                    row.focus()
                    row.press('ContextMenu' if method == 'menu' else 'Shift+F10')
                expect(page.locator('[data-command=directory-expand]')).to_be_visible()
            except Exception as error:
                failure = {'status':'failed','stage':'open_directory_menu','context':context,
                           'before_input':before,'page_errors':list(errors),
                           'original_exception_type':type(error).__name__,'passed_report_written':False}
                try:
                    failure['after_failure'] = row.evaluate('(row)=>__xdzipMenuDiagnostics.snapshot(row)')
                    page.screenshot(path=str(OUTPUT / 'directory-menu-failure.png'))
                    failure['owned_html_screenshot'] = 'directory-menu-failure.png'
                except Exception as diagnostic_error:
                    failure['diagnostic_exception_type'] = type(diagnostic_error).__name__
                try:
                    (OUTPUT / 'directory-menu-failure.json').write_text(
                        json.dumps(failure,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
                    print(json.dumps(failure,ensure_ascii=False),flush=True)
                except Exception:
                    pass  # Diagnostic failure must not replace the original assertion.
                raise

        def assert_description_only(owner, command, reason, hover=False):
            expect(command).to_have_attribute('aria-disabled', 'true')
            expect(command).to_have_attribute('aria-description', reason)
            expect(owner.locator('.ui-menu-reason')).to_have_count(0)
            assert reason not in command.inner_text()
            assert command.get_attribute('data-tooltip') is None
            assert command.get_attribute('title') is None
            if hover:
                command.hover()
                owner.wait_for_timeout(owner.evaluate('XDZIP_CONTROL_TOKENS.motion.hoverDelayMs') + 80)
                expect(owner.locator('[role=tooltip]')).to_have_count(0)

        for language, root_reason, leaf_reason, collapse in [
            ('zh-CN', '根目录始终展开', '该节点没有子目录', '收起子目录'),
            ('en-US', 'Root is always expanded', 'This entry has no subfolders', 'Collapse subfolders'),
        ]:
            page.evaluate('(language)=>document.documentElement.lang=language', language)
            for text_scale in (1, 1.25, 1.5):
                page.evaluate('(scale)=>document.documentElement.style.setProperty("--settings-text-scale",scale)', text_scale)
                for method in ['mouse', 'menu', 'shift-f10']:
                    open_directory_menu(page.locator('.tree-root[data-directory=""]'), method,
                                        language=language, text_scale=text_scale)
                    command = page.locator('[data-command=directory-expand]')
                    assert_description_only(page, command, root_reason, method == 'mouse')
                    expect(command.locator('.ui-menu-label')).to_have_text(collapse)
                    divider = page.locator('.ui-menu [role=separator]')
                    expect(divider).to_be_visible()
                    assert divider.evaluate('(n)=>n.getBoundingClientRect().height>0 && getComputedStyle(n).backgroundColor!=="rgba(0, 0, 0, 0)"')
                    command.click(force=True)
                    expect(page.locator('.ui-menu')).to_have_count(1)
                    assert page.evaluate('({path:state.path,selected:[...state.selected],expanded:[...state.expanded]})') == tree_snapshot
                    page.keyboard.press('Escape')
                    for expanded in [False, True]:
                        open_directory_menu(page.locator('[data-directory=photos]'), method,
                                            language=language, text_scale=text_scale)
                        command = page.locator('[data-command=directory-expand]')
                        expect(command).to_have_attribute('aria-disabled', 'false')
                        assert command.get_attribute('aria-description') is None
                        command.click()
                        expect(page.locator('[data-directory=photos]')).to_have_attribute('aria-expanded', str(not expanded).lower())
                    open_directory_menu(page.locator('[data-directory=maps]'), method,
                                        language=language, text_scale=text_scale)
                    command = page.locator('[data-command=directory-expand]')
                    assert_description_only(page, command, leaf_reason, method == 'mouse')
                    command.click(force=True)
                    assert page.evaluate('({path:state.path,selected:[...state.selected],expanded:[...state.expanded]})') == tree_snapshot
                    directory_cases.append({'language':language,'text_scale':text_scale,'method':method,
                                            'root_and_leaf_description_only':True,'parent_toggles':True,'divider_visible':True})
                    page.keyboard.press('Escape')
        page.evaluate('document.documentElement.lang="zh-CN"')
        page.evaluate('document.documentElement.style.setProperty("--settings-text-scale",1)')
        checks.append('Directory root, parent and leaf: pointer/Menu/Shift+F10 preserve bilingual reasons only as accessible descriptions, without hints or tooltips at three text scales')

        # A real scroll position change during the preceding input queues a
        # browser scroll event. Its delivery must not close the new menu when
        # the position is already the one recorded as the menu opened.
        original_table_style = page.locator('.table-scroll table').get_attribute('style')
        page.evaluate("""() => {
            const root=document.querySelector('.tree-root[data-directory=""]');
            const scroller=document.querySelector('.table-scroll'), table=scroller.querySelector('table');
            table.style.minWidth='2400px'; scroller.scrollLeft=0;
            const state=window.__xdzipQueuedScroll={input_count:0,scrolls:[],opened:null,phase:'preceding_input'};
            root.addEventListener('contextmenu',()=>{
                state.input_count++; state.before_capture=scroller.scrollLeft;
                scroller.scrollLeft=32; state.after_capture=scroller.scrollLeft;
            },{capture:true,once:true});
            const opened=event=>{
                if (!root.contains(event.target)) return;
                state.opened={position:scroller.scrollLeft,menu_count:document.querySelectorAll('.ui-menu').length,
                    scroller_is_trigger_ancestor:scroller.contains(root)};
                document.removeEventListener('contextmenu',opened);
            };
            document.addEventListener('contextmenu',opened);
            state.onScroll=event=>{
                if(event.target===scroller) state.scrolls.push({phase:state.phase,position:scroller.scrollLeft,
                    trusted:event.isTrusted,menu_count:document.querySelectorAll('.ui-menu').length});
            };
            document.addEventListener('scroll',state.onScroll,true);
        }""")
        assert page.locator('.table-scroll').evaluate('(node)=>node.scrollWidth>node.clientWidth && node.scrollLeft===0')
        open_directory_menu(page.locator('.tree-root[data-directory=""]'), 'mouse', language='zh-CN', text_scale=1,
                            scenario='queued_nonancestor_scroll_before_menu_open')
        page.wait_for_function('__xdzipQueuedScroll.scrolls.length>0')
        queued = page.evaluate('JSON.parse(JSON.stringify(__xdzipQueuedScroll))')
        queued_observations = {'status':'observed','preceding_input':queued}
        (OUTPUT / 'queued-scroll-observations.json').write_text(
            json.dumps(queued_observations,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
        assert queued['input_count'] == 1 and queued['before_capture'] == 0 and queued['after_capture'] == 32
        assert queued['opened'] == {'position':32,'menu_count':1,'scroller_is_trigger_ancestor':False}
        assert all(event == {'phase':'preceding_input','position':32,'trusted':True,'menu_count':1} for event in queued['scrolls'])
        expect(page.locator('[data-command=directory-expand]')).to_be_visible()
        queued_scroll_cases.append({'phase':'preceding_input','position_at_open':32,'event_position':32,
                                    'browser_trusted_scroll':True,'menu_visible_after_event':True})
        checks.append('A queued browser scroll from the preceding real contextmenu input preserves the menu at its already-recorded non-ancestor position')
        page.evaluate("""() => {
            __xdzipQueuedScroll.phase='after_open';
            document.querySelector('.table-scroll').scrollLeft=64;
        }""")
        page.wait_for_function("__xdzipQueuedScroll.scrolls.some(event=>event.phase==='after_open')")
        queued = page.evaluate('JSON.parse(JSON.stringify(__xdzipQueuedScroll))')
        queued_observations['after_open'] = queued
        (OUTPUT / 'queued-scroll-observations.json').write_text(
            json.dumps(queued_observations,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
        changed = [event for event in queued['scrolls'] if event['phase'] == 'after_open']
        assert changed and all(event == {'phase':'after_open','position':64,'trusted':True,'menu_count':0} for event in changed)
        expect(page.locator('.ui-menu')).to_have_count(0)
        queued_scroll_cases.append({'phase':'after_open','position_at_open':32,'event_position':64,
                                    'browser_trusted_scroll':True,'menu_closed_after_event':True})
        checks.append('A real post-open non-ancestor scroll position change closes the menu')
        page.evaluate("""original => {
            document.removeEventListener('scroll',__xdzipQueuedScroll.onScroll,true);
            const table=document.querySelector('.table-scroll table');
            if(original===null) table.removeAttribute('style'); else table.setAttribute('style',original);
            document.querySelector('.table-scroll').scrollLeft=0;
        }""", original_table_style)
        for ident in ['open', 'create', 'test-archive', 'more-actions', 'extract', 'extract-menu-button']:
            assert page.locator('#' + ident).get_attribute('data-tooltip') is None
            assert page.locator('#' + ident).get_attribute('title') is None
        checks.append('Main toolbar actions have no hover tooltips or browser titles')


        hint = page.locator('#new-tab')
        hint.hover()
        page.wait_for_timeout(700)
        expect(page.locator('[role=tooltip]')).to_have_count(0)
        bounds = hint.bounding_box()
        page.mouse.move(bounds['x'] + bounds['width'] / 2 + 3, bounds['y'] + bounds['height'] / 2)
        page.wait_for_timeout(700)
        expect(page.locator('[role=tooltip]')).to_have_count(0)
        expect(page.locator('[role=tooltip]')).to_be_visible(timeout=1800)
        page.mouse.down()
        try:
            expect(page.locator('[role=tooltip]')).to_have_count(0)
            page.wait_for_timeout(1300)
            expect(page.locator('[role=tooltip]')).to_have_count(0)
        finally:
            page.mouse.move(1, 1)
            page.mouse.up()
        page.mouse.move(1, 1)
        expect(page.locator('[role=tooltip]')).to_have_count(0)
        hint.hover()
        page.wait_for_timeout(700)
        expect(page.locator('[role=tooltip]')).to_have_count(0)
        page.mouse.move(1, 1)
        page.wait_for_timeout(600)
        expect(page.locator('[role=tooltip]')).to_have_count(0)
        checks.append('Tooltip requires pointer rest, resets after movement/re-entry and stays hidden on press/leave')

        page.locator('#more-actions').focus()
        assert page.locator('#more-actions').evaluate('el=>getComputedStyle(el).outlineStyle') == 'none'
        page.locator('.app-brand').click()
        expect(page.locator('#more-actions')).not_to_be_focused()
        page.locator('#search').focus()
        page.keyboard.press('Enter')
        expect(page.locator('#search')).not_to_be_focused()
        checks.append('Controls have no focus outline and lose focus on outside click or Enter')

        page.locator('#tasks').click()
        expect(page.locator('[data-command=no-tasks]')).to_be_visible()
        page.keyboard.press('Escape')
        expect(page.locator('.ui-menu')).to_have_count(0)
        expect(page.locator('#tasks')).not_to_be_focused()
        checks.append('Escape closes an all-disabled task menu and dismisses its trigger focus')

        page.locator('#more-actions').click()
        expect(page.locator('.ui-menu')).to_have_count(1)
        expect(page.locator('#more-actions')).to_have_attribute('aria-expanded', 'true')
        page.keyboard.press('End')
        expect(page.locator('[data-command=settings]')).to_be_focused()
        page.keyboard.press('Home')
        expect(page.locator('[data-command=settings]')).to_be_focused()
        expect(page.locator('.ui-menu [role=menuitem]')).to_have_count(1)
        page.keyboard.press('Escape')
        expect(page.locator('#more-actions')).not_to_be_focused()
        page.locator('#file-rows [data-id=lake]').click(button='right')
        page.locator('[data-command=view-menu]').focus()
        page.keyboard.press('ArrowRight')
        expect(page.locator('.ui-menu')).to_have_count(2)
        expect(page.locator('[data-command=view-list]')).to_have_attribute('aria-checked', 'true')
        page.keyboard.press('ArrowLeft')
        expect(page.locator('.ui-menu')).to_have_count(1)
        expect(page.locator('[data-command=view-menu]')).to_be_focused()
        page.keyboard.press('Escape')
        checks.append('More menu contains settings only; context submenu preserves Home/End, arrows, checked state and Escape')

        expect(page.locator('.column-resize-handle')).to_have_count(5)
        before = page.locator('th:nth-child(3)').bounding_box()['width']
        original_sort = page.evaluate('({sort:state.sort,key:state.sortKey})')
        handle = page.locator('.column-resize-handle[data-column="1"]')
        boundary = handle.bounding_box()
        page.mouse.move(boundary['x'] + boundary['width']/2, boundary['y'] + boundary['height']/2)
        page.mouse.down()
        page.mouse.move(boundary['x'] + boundary['width']/2 + 35, boundary['y'] + boundary['height']/2)
        page.mouse.up()
        assert page.locator('th:nth-child(3)').bounding_box()['width'] > before + 30
        assert page.evaluate('({sort:state.sort,key:state.sortKey})') == original_sort
        alignment = page.evaluate('''() => [...document.querySelectorAll('thead th')].map((header,index)=>Math.abs(header.getBoundingClientRect().width-document.querySelector('#file-rows tr').children[index].getBoundingClientRect().width))''')
        assert max(alignment) < 1
        handle.dblclick()
        assert abs(page.locator('th:nth-child(3)').bounding_box()['width'] - before) < 1
        handle.focus()
        page.keyboard.press('ArrowRight')
        assert page.locator('th:nth-child(3)').bounding_box()['width'] > before + 6
        page.keyboard.press('Home')
        assert abs(page.locator('th:nth-child(3)').bounding_box()['width'] - before) < 1
        checks.append('Five resize boundaries drag without sorting, align rows, support keyboard and reset to defaults')
        geometry = page.evaluate('''() => {
            const frame=document.querySelector('.window'), saved=frame.style.width;
            const factor=parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--ui-source-factor'));
            const results=[];
            for (const width of [832,928,1024,1440]) {
                frame.style.width=width*factor+'px';
                const buttons=[...document.querySelectorAll('.file-tools .icon-button')].filter(button=>button.getBoundingClientRect().width>0);
                results.push(...buttons.map(button=>button.getBoundingClientRect().width/factor));
                results.push(...buttons.slice(1).map((button,index)=>(button.getBoundingClientRect().left-buttons[index].getBoundingClientRect().right)/factor));
            }
            frame.style.width=saved;
            return results;
        }''')
        assert all(abs(value-28)<.1 or abs(value-6.4)<.1 for value in geometry)
        checks.append('File action buttons stay 28 units with 6.4-unit gaps at narrow and wide window widths')

        page.locator('#extract-menu-button').click()
        expect(page.locator('[data-command=extract-results]')).to_have_attribute('aria-disabled', 'true')
        assert_description_only(page, page.locator('[data-command=extract-results]'), '仅有搜索结果时可用', True)
        page.locator('[data-command=extract-results]').click(force=True)
        expect(page.locator('.ui-menu')).to_have_count(1)
        expect(page.locator('#modal')).not_to_be_visible()
        page.keyboard.press('End')
        expect(page.locator('[data-command=extract-selected]')).to_be_focused()
        page.keyboard.press('Escape')
        checks.append('Unavailable search-result action cannot execute and is skipped by keyboard')

        page.locator('#file-rows [data-id=sunrise]').click(modifiers=['ControlOrMeta'])
        expect(page.locator('#selection-status')).to_contain_text('2 项')
        page.locator('#file-rows [data-id=lake]').click(button='right')
        assert set(page.evaluate('[...state.selected]')) == {'lake', 'sunrise'}
        expect(page.locator('[data-command=copy-name]')).to_be_visible()
        page.screenshot(path=str(OUTPUT / 'main-context-light.png'))
        page.keyboard.press('Escape')
        expect(page.locator('#file-rows [data-id=lake]')).not_to_be_focused()
        page.locator('#file-rows [data-id=guide]').click(button='right')
        assert page.evaluate('[...state.selected]') == ['guide']
        page.keyboard.press('Escape')
        expect(page.locator('.ui-menu')).to_have_count(0)
        expect(page.locator('#file-rows [data-id=guide]')).not_to_be_focused()
        page.locator('#file-rows [data-id=guide]').press('Shift+F10')
        expect(page.locator('.ui-menu')).to_have_count(1)
        page.locator('[data-command=copy-path]').click()
        expect(page.locator('#copy-text')).to_have_value('行程手册.pdf')
        page.locator('#modal [data-close]').click()
        expect(page.locator('#file-rows [data-id=guide]')).to_be_focused()
        checks.append('Context targets: preserve multi-selection, replace unselected target, keyboard entry, copy fallback')

        page.keyboard.press('Control+A')
        assert len(page.evaluate('[...state.selected]')) == 9
        page.keyboard.press('Control+Shift+A')
        assert page.evaluate('[...state.selected]') == []
        page.locator('#file-rows [data-id=lake]').click()
        page.locator('#file-rows [data-id=list]').click(modifiers=['Shift'])
        assert set(page.evaluate('[...state.selected]')) == {'lake', 'sunrise', 'guide', 'list'}
        page.locator('#file-rows [data-id=list]').press('Control+ArrowDown')
        expect(page.locator('#file-rows [data-id=route]')).to_be_focused()
        assert len(page.evaluate('[...state.selected]')) == 4
        page.locator('#file-rows [data-id=route]').press('Space')
        assert len(page.evaluate('[...state.selected]')) == 5
        checks.append('Selection: all/none, range, focus-only arrow and Space preserve the intended set')

        page.locator('#grid-view').click()
        expect(page.locator('#file-grid .grid-entry')).to_have_count(9)
        expect(page.locator('#grid-view')).to_have_attribute('aria-pressed', 'true')
        page.locator('#file-grid [data-id=lake]').click(button='right')
        expect(page.locator('.ui-menu')).to_have_count(1)
        page.keyboard.press('Escape')
        expect(page.locator('#file-grid [data-id=lake]')).not_to_be_focused()
        page.locator('#list-view').click()
        page.locator('th:nth-child(3)').click(button='right')
        page.locator('[data-command=sort-size]').click()
        expect(page.locator('th:nth-child(3)')).to_have_attribute('aria-sort', 'ascending')
        page.locator('th:nth-child(3) button').click()
        expect(page.locator('th:nth-child(3)')).to_have_attribute('aria-sort', 'descending')
        checks.append('List/grid share entries and selection; header context and click share sorting state')

        # Owned extensionless records must not share the directory group.
        page.evaluate('''() => {
            for (let i=0;i<2;i++) {
                const entry={id:`type-regression-${i}`,parent:'',name:'LICENSE',type:'文件',size:1,compressed:1,modified:'2026/10/13'};
                files.push(entry); entries.push(entry); byId.set(entry.id,entry);
            }
            render();
        }''')
        for direction in ['ascending', 'descending']:
            page.locator('[data-sort=type]').click()
            expect(page.locator('th:nth-child(5)')).to_have_attribute('aria-sort', direction)
            rows = page.evaluate('''() => [...document.querySelectorAll('#file-rows tr')].map(row=>({id:row.dataset.id,folder:Boolean(byId.get(row.dataset.id).folder)}))''')
            assert [row['id'] for row in rows[:3]] == ['photos', 'documents', 'maps']
            assert all(not row['folder'] for row in rows[3:])
            assert [row['id'] for row in rows if row['id'].startswith('type-regression-')] == ['type-regression-0', 'type-regression-1']
        page.evaluate('''() => {
            files.splice(files.length-2,2); entries.splice(entries.length-2,2);
            byId.delete('type-regression-0'); byId.delete('type-regression-1'); render();
        }''')
        checks.append('Type header: directories precede extensionless files in both directions; duplicate ties keep identity order')

        saved = page.evaluate('({selected:[...state.selected],sort:state.sort,sortKey:state.sortKey})')
        page.locator('[data-tab=project]').click()
        expect(page.locator('#file-rows tr')).to_have_count(2)
        expect(page.locator('#selection-status')).not_to_contain_text('已选择')
        page.locator('#search').fill('贡献')
        expect(page.locator('#file-rows tr')).to_have_count(4)
        page.locator('[data-tab=travel]').click()
        assert page.evaluate('({selected:[...state.selected],sort:state.sort,sortKey:state.sortKey})') == saved
        expect(page.locator('#search')).to_have_value('')
        page.locator('[data-tab=project]').click()
        expect(page.locator('#search')).to_have_value('贡献')
        page.locator('#extract').click()
        expect(page.locator('#modal .modal-header p')).to_contain_text('项目素材.zip')
        expect(page.locator('#target-preview')).to_have_text('最终位置：下载 / 项目素材')
        page.keyboard.press('Escape')
        expect(page.locator('#modal')).to_be_visible()
        page.keyboard.press('Escape')
        expect(page.locator('#modal')).not_to_be_visible()
        page.locator('[data-tab=project]').click(button='right')
        page.locator('[data-command=tab-close]').click()
        expect(page.locator('[data-tab]')).to_have_count(1)
        page.keyboard.press('Control+Shift+T')
        expect(page.locator('[data-tab]')).to_have_count(2)
        expect(page.locator('#search')).to_have_value('贡献')
        page.locator('[data-tab=travel]').click()
        checks.append('Archive sessions isolate query, selection and sort; closing/reopening restores the session')

        page.locator('[data-toggle=photos]').click()
        expect(page.locator('[data-directory=ice]')).to_be_visible()
        assert page.evaluate('state.path') == ''
        page.locator('[data-toggle=ice]').click()
        page.locator('[data-directory=ice]').click()
        expect(page.locator('#file-rows tr')).to_have_count(2)
        expect(page.locator('[data-directory=snow] .tree-disclosure')).to_be_disabled()
        page.locator('[data-toggle=photos]').click()
        assert page.evaluate('state.path') == 'ice'
        expect(page.locator('[data-directory=photos] .tree-branch-marker')).to_be_visible()
        expect(page.locator('[data-directory=photos]')).to_have_attribute('aria-selected', 'false')
        page.locator('#breadcrumbs [data-breadcrumb=photos]').click()
        assert page.evaluate('state.path') == 'photos'
        page.locator('[data-directory=photos]').focus()
        page.keyboard.press('ArrowRight')
        expect(page.locator('[data-directory=ice]')).to_be_visible()
        page.keyboard.press('ArrowRight')
        expect(page.locator('[data-directory=ice]')).to_be_focused()
        page.keyboard.press('Shift+F10')
        page.locator('[data-command=directory-search]').click()
        assert page.evaluate('state.path') == 'ice'
        expect(page.locator('#search')).to_be_focused()
        checks.append('Tree: independent disclosure, real child arrows, collapsed current marker, breadcrumbs and keyboard context')

        page.locator('#breadcrumbs [data-home]').click()
        page.keyboard.press('Control+F')
        page.locator('#search').dispatch_event('compositionstart')
        page.locator('#search').fill('行程')
        assert page.evaluate('state.query') == ''
        page.locator('#search').dispatch_event('compositionend')
        expect(page.locator('#file-rows tr')).to_have_count(1)
        page.locator('#more-actions').click()
        page.locator('#search').fill('冰川湖')
        expect(page.locator('.ui-menu')).to_have_count(0)
        search_count = page.locator('#file-rows tr').count()
        page.locator('#search').press('Escape')
        expect(page.locator('#search')).not_to_be_focused()
        expect(page.locator('#search')).to_have_value('冰川湖')
        expect(page.locator('#file-rows tr')).to_have_count(search_count)
        assert page.evaluate('state.query') == '冰川湖'
        page.locator('#search').fill('行程')
        expect(page.locator('#search-clear')).to_be_visible()
        page.locator('#search-clear').click()
        expect(page.locator('#search')).to_have_value('')
        expect(page.locator('#search')).to_be_focused()
        expect(page.locator('#file-rows tr')).to_have_count(9)
        checks.append('Search: IME composition, old-menu invalidation, Escape preserves text and clear-button focus return')

        page.locator('#test-archive').click()
        expect(page.locator('#modal')).to_have_attribute('data-modeless', 'true')
        expect(page.locator('#task-count')).to_have_text('任务 1')
        expect(page.locator('#test-archive')).to_be_disabled()
        page.locator('#close-archive-test').click()
        expect(page.locator('#modal')).not_to_be_visible()
        page.locator('#tasks').click()
        page.locator('[data-command=task-details]').click()
        expect(page.locator('#modal .modal-footer')).to_contain_text('不执行实际校验')
        expect(page.locator('#modal .progress-track')).to_have_attribute('role', 'progressbar')
        page.locator('#cancel-archive-test').click()
        expect(page.locator('#modal h1')).to_contain_text('已取消')
        expect(page.locator('#task-count')).to_have_text('任务 1')
        page.locator('#close-archive-test').click()
        expect(page.locator('#test-archive')).to_be_enabled()
        checks.append('Demo integrity job: busy guard, modeless close keeps running, reopen and explicit cancellation')

        page.locator('#more-actions').hover()
        page.wait_for_timeout(500)
        expect(page.locator('[role=tooltip]')).to_have_count(0)
        page.locator('#more-actions').click()
        expect(page.locator('[role=tooltip]')).to_have_count(0)
        page.locator('#list-view').click()
        expect(page.locator('.ui-menu')).to_have_count(0)
        page.locator('#more-actions').click()
        page.keyboard.press('Tab')
        expect(page.locator('.ui-menu')).to_have_count(0)
        assert page.evaluate('!document.activeElement.closest(".ui-menu")')
        checks.append('No toolbar hover tooltip; outside action and Tab release the menu without trapping focus')

        # Release after a real Chromium touch hold must open, not execute, the menu.
        lake = page.locator('#file-rows [data-id=lake]').bounding_box()
        session = context.new_cdp_session(page)
        point = {'x':lake['x']+100,'y':lake['y']+lake['height']/2}
        session.send('Input.dispatchTouchEvent', {'type':'touchStart','touchPoints':[point]})
        expect(page.locator('.ui-menu')).to_have_count(1, timeout=2000)
        session.send('Input.dispatchTouchEvent', {'type':'touchEnd','touchPoints':[]})
        expect(page.locator('.ui-menu')).to_have_count(1)
        expect(page.locator('#modal')).not_to_be_visible()
        page.keyboard.press('Escape')
        checks.append('Touch long press opens the same context; finger release cannot execute its first action')

        for platform in ('macos', 'windows', 'linux'):
            page.locator('#platform').select_option(platform, force=True)
            for theme in ('light', 'dark'):
                page.evaluate('(theme) => XdzipDesign.theme(theme)', theme)
                page.locator('#more-actions').click()
                expect(page.locator('.ui-menu')).to_be_visible()
                page.screenshot(path=str(OUTPUT / f'main-more-{platform}-{theme}.png'))
                page.keyboard.press('Escape')
                checks.append(f'{platform}/{theme}: live main menu and themed icons render')

        gallery = context.new_page()
        gallery.on('pageerror', lambda error: errors.append(str(error)))
        gallery.set_content((ROOT / 'dist/xdzip-components.html').read_text())
        gallery.evaluate('document.fonts.ready')
        gallery.set_viewport_size({'width':1588,'height':1240})
        for theme in ('light', 'dark'):
            if gallery.evaluate('document.documentElement.dataset.theme') != theme:
                gallery.locator('#component-theme').click()
            gallery.locator('#component-more').click()
            expect(gallery.locator('.ui-menu')).to_be_visible()
            assert_description_only(gallery, gallery.locator('[data-command=catalog-extract]'), '设计示例不会写出文件', True)
            assert_description_only(gallery, gallery.locator('[data-command=catalog-test]'), '设计示例不执行真实校验', True)
            gallery.screenshot(path=str(OUTPUT / f'components-{theme}.png'))
            gallery.keyboard.press('Escape')
        button=gallery.locator('#component-open')
        initial=button.bounding_box()
        button.hover()
        gallery.mouse.down()
        assert button.bounding_box() == initial
        gallery.mouse.up()
        expect(gallery.locator('#component-notice')).to_contain_text('打开动作')
        gallery.locator('#component-remove').click()
        expect(gallery.locator('#component-context')).to_be_visible()
        expect(gallery.locator('#component-cancel-remove')).to_be_focused()
        gallery.locator('#component-cancel-remove').click()
        expect(gallery.locator('#component-context')).to_be_visible()
        gallery.locator('#component-remove').click()
        gallery.locator('#component-confirm-remove').click()
        expect(gallery.locator('#component-context')).not_to_be_visible()
        gallery.locator('#component-restore').click()
        expect(gallery.locator('#component-context')).to_be_visible()
        checks.append('Component actions keep button geometry stable and require explicit removable-demo confirmation')
        gallery.locator('#component-context').press('Shift+F10')
        gallery.locator('[data-command=catalog-view]').focus()
        gallery.keyboard.press('ArrowRight')
        gallery.locator('[data-command=catalog-preview]').click()
        expect(gallery.locator('#component-inspector')).not_to_be_checked()
        gallery.locator('#component-sort').click()
        expect(gallery.locator('[data-command=catalog-sort-name]')).to_have_attribute('aria-checked', 'true')
        gallery.locator('[data-command=catalog-sort-size]').click()
        gallery.locator('#component-sort').click()
        expect(gallery.locator('[data-command=catalog-sort-size]')).to_have_attribute('aria-checked', 'true')
        gallery.keyboard.press('Escape')
        checks.append('Component catalog uses live shared check and radio items')
        for width in (390, 320):
            gallery.set_viewport_size({'width':width,'height':844})
            gallery.locator('#component-edge').click()
            bounds = gallery.locator('.ui-menu').bounding_box()
            assert bounds['x'] >= 0 and bounds['y'] >= 0
            assert bounds['x']+bounds['width'] <= width and bounds['y']+bounds['height'] <= 844
            assert gallery.evaluate('document.documentElement.scrollWidth <= innerWidth')
            gallery.screenshot(path=str(OUTPUT / f'components-mobile-{width}.png'))
            gallery.keyboard.press('Escape')
            checks.append(f'{width}px mobile: live menu flips at viewport edge; catalog has no horizontal overflow')
        assert not errors, errors
        assert not requests, requests
        report = {'status':'passed','checks':checks,'check_count':len(checks),'page_errors':errors,'external_requests':requests,
                  'directory_menu_cases':directory_cases,
                  'queued_scroll_cases':queued_scroll_cases,
                  'limits':['HTML operations use in-memory example data','Platform decorations in HTML do not prove native window behavior',
                            'Qt and assistive-technology validation are separate']}
        (OUTPUT / 'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
        print(json.dumps(report,ensure_ascii=False))
        browser.close()


if __name__ == '__main__':
    main()
