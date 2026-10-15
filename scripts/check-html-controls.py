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
OUTPUT = ROOT / 'dist/html-controls-review'
OUTPUT.mkdir(parents=True, exist_ok=True)


def main():
    checks, errors, requests = [], [], []
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
        page.set_content(canvas)
        page.evaluate('document.fonts.ready')
        expect(page.locator('#file-rows tr')).to_have_count(9)
        expect(page.locator('[data-tab]')).to_have_count(2)
        expect(page.locator('#tree [role=treeitem]')).to_have_count(4)
        checks.append('Initial archive, table and directory controls load offline')
        for ident in ['open', 'create', 'test-archive', 'more-actions', 'extract', 'extract-menu-button']:
            assert page.locator('#' + ident).get_attribute('data-tooltip') is None
            assert page.locator('#' + ident).get_attribute('title') is None
        checks.append('Main toolbar actions have no hover tooltips or browser titles')


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
        expect(page.locator('[data-command=archive-properties]')).to_be_focused()
        page.locator('[data-command=more-view]').focus()
        page.keyboard.press('ArrowRight')
        expect(page.locator('.ui-menu')).to_have_count(2)
        expect(page.locator('[data-command=view-list]')).to_have_attribute('aria-checked', 'true')
        page.keyboard.press('ArrowLeft')
        expect(page.locator('.ui-menu')).to_have_count(1)
        expect(page.locator('[data-command=more-view]')).to_be_focused()
        page.keyboard.press('Escape')
        expect(page.locator('#more-actions')).not_to_be_focused()
        checks.append('More menu: Home/End, submenu arrows, checked state, Escape and focus dismissal')

        page.locator('#extract-menu-button').click()
        expect(page.locator('[data-command=extract-results]')).to_have_attribute('aria-disabled', 'true')
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
                  'limits':['HTML operations use in-memory example data','Platform decorations in HTML do not prove native window behavior',
                            'Qt and assistive-technology validation are separate']}
        (OUTPUT / 'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
        print(json.dumps(report,ensure_ascii=False))
        browser.close()


if __name__ == '__main__':
    main()
