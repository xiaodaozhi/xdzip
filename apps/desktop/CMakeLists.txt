find_package(Qt6 6.8 REQUIRED COMPONENTS Core Gui Qml Quick QuickControls2)
qt_standard_project_setup()
qt_policy(SET QTP0001 NEW)
qt_add_resources(ui_resources ui.qrc)
add_library(xdzip-ui-resources STATIC ${ui_resources})
qt_add_qml_module(xdzip-ui-resources
    URI Xdzip.Settings VERSION 1.0
    OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/qml/Xdzip/Settings"
    SOURCES UiPreferences.h UiPreferences.cpp)
target_link_libraries(xdzip-ui-resources PRIVATE Qt6::Core Qt6::Gui Qt6::Qml)
add_executable(xdzip-ui-preview main.cpp)
target_link_libraries(xdzip-ui-preview PRIVATE Qt6::Quick Qt6::QuickControls2)
if(BUILD_TESTING)
    add_test(NAME qt_preview_load COMMAND xdzip-ui-preview --smoke-test)
    find_package(Qt6 6.8 REQUIRED COMPONENTS QuickTest)
    add_executable(xdzip-ui-tests tests/main.cpp)
    target_link_libraries(xdzip-ui-tests PRIVATE Qt6::QuickTest Qt6::QuickControls2)
    add_test(NAME qt_task_components COMMAND xdzip-ui-tests -input "${CMAKE_CURRENT_SOURCE_DIR}/tests")
    set_tests_properties(qt_task_components PROPERTIES TIMEOUT 30)
endif()

foreach(target xdzip-ui-resources xdzip-ui-preview xdzip-ui-tests)
    if(TARGET ${target})
        if(MSVC)
            target_compile_options(${target} PRIVATE /W4 /WX /utf-8)
        else()
            target_compile_options(${target} PRIVATE -Wall -Wextra -Werror)
        endif()
    endif()
endforeach()

find_package(Qt6 6.8 REQUIRED COMPONENTS Widgets)
include(${PROJECT_SOURCE_DIR}/cmake/RustStaticLibrary.cmake)
xdzip_rust_library(xdzip-reader-rust xdzip-qt-bridge xdzip_qt_bridge)
qt_add_executable(xdzip-browser browser_main.cpp)
set(XDZIP_BRAND_ASSETS "${PROJECT_SOURCE_DIR}/design/assets/brand")
if(WIN32)
    # Only the product uses the GUI subsystem; test/probe consoles retain diagnostics.
    set_target_properties(xdzip-browser PROPERTIES WIN32_EXECUTABLE TRUE)
    enable_language(RC)
    set(XDZIP_APP_ICON_ICO "${XDZIP_BRAND_ASSETS}/xdzip.ico")
    configure_file(packaging/windows/app.rc.in "${CMAKE_CURRENT_BINARY_DIR}/xdzip.rc" @ONLY)
    target_sources(xdzip-browser PRIVATE "${CMAKE_CURRENT_BINARY_DIR}/xdzip.rc")
endif()
if(APPLE)
    enable_language(OBJCXX)
    set_target_properties(xdzip-browser PROPERTIES OBJCXX_STANDARD 17
        OBJCXX_STANDARD_REQUIRED ON OBJCXX_EXTENSIONS OFF)
    target_sources(xdzip-browser PRIVATE MacTitlebar.mm)
    target_link_libraries(xdzip-browser PRIVATE "-framework AppKit")
    set_target_properties(xdzip-browser PROPERTIES MACOSX_BUNDLE TRUE OUTPUT_NAME xdzip
        MACOSX_BUNDLE_GUI_IDENTIFIER "org.xdzip.desktop"
        MACOSX_BUNDLE_INFO_PLIST "${CMAKE_CURRENT_SOURCE_DIR}/packaging/macos/Info.plist.in"
        MACOSX_BUNDLE_BUNDLE_NAME "xdzip"
        MACOSX_BUNDLE_ICON_FILE "xdzip.icns"
        MACOSX_BUNDLE_SHORT_VERSION_STRING "${PROJECT_VERSION}"
        MACOSX_BUNDLE_BUNDLE_VERSION "${PROJECT_VERSION}")
    set_source_files_properties("${XDZIP_BRAND_ASSETS}/xdzip.icns" PROPERTIES
        MACOSX_PACKAGE_LOCATION "Resources")
    target_sources(xdzip-browser PRIVATE "${XDZIP_BRAND_ASSETS}/xdzip.icns")
endif()
qt_add_qml_module(xdzip-browser
    URI Xdzip.Native VERSION 1.0
    DEPENDENCIES QtQml.Models
    OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/qml/Xdzip/Native"
    SOURCES BrowserController.h BrowserController.cpp RuntimeConnection.h RuntimeConnection.cpp)
target_link_libraries(xdzip-browser PRIVATE xdzip-reader-rust
    Qt6::Widgets Qt6::Quick Qt6::QuickControls2)
if(UNIX AND NOT APPLE)
    include(GNUInstallDirs)
    install(TARGETS xdzip-browser RUNTIME DESTINATION "${CMAKE_INSTALL_BINDIR}")
    install(FILES packaging/linux/xdzip.desktop DESTINATION "${CMAKE_INSTALL_DATADIR}/applications")
    install(FILES "${XDZIP_BRAND_ASSETS}/xdzip.png"
        DESTINATION "${CMAKE_INSTALL_DATADIR}/icons/hicolor/512x512/apps")
endif()
if(BUILD_TESTING)
    add_executable(xdzip-browser-tests tests/browser_test.cpp BrowserController.cpp BrowserController.h RuntimeConnection.cpp RuntimeConnection.h)
    target_include_directories(xdzip-browser-tests PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
    find_package(Python3 REQUIRED COMPONENTS Interpreter)
    set(XDZIP_M2_FIXTURE "${CMAKE_CURRENT_BINARY_DIR}/m2-flat.zip")
    add_custom_command(OUTPUT "${XDZIP_M2_FIXTURE}"
        COMMAND Python3::Interpreter "${PROJECT_SOURCE_DIR}/scripts/generate-m2-fixture.py" --output "${XDZIP_M2_FIXTURE}"
        DEPENDS "${PROJECT_SOURCE_DIR}/scripts/generate-m2-fixture.py" VERBATIM)
    set(XDZIP_M2_HIERARCHICAL "${CMAKE_CURRENT_BINARY_DIR}/m2-hierarchical.zip")
    add_custom_command(OUTPUT "${XDZIP_M2_HIERARCHICAL}"
        COMMAND Python3::Interpreter "${PROJECT_SOURCE_DIR}/scripts/generate-m2-fixture.py" --output "${XDZIP_M2_HIERARCHICAL}" --hierarchical
        DEPENDS "${PROJECT_SOURCE_DIR}/scripts/generate-m2-fixture.py" VERBATIM)
    add_custom_target(xdzip-m2-fixtures DEPENDS "${XDZIP_M2_FIXTURE}" "${XDZIP_M2_HIERARCHICAL}")
    add_dependencies(xdzip-browser-tests xdzip-m2-fixtures)
    target_compile_definitions(xdzip-browser-tests PRIVATE XDZIP_FIXTURES="${PROJECT_SOURCE_DIR}/tests/fixtures/zip" XDZIP_M2_FIXTURE="${XDZIP_M2_FIXTURE}" XDZIP_M2_HIERARCHICAL="${XDZIP_M2_HIERARCHICAL}")
    target_link_libraries(xdzip-browser-tests PRIVATE xdzip-reader-rust
        Qt6::Widgets Qt6::Quick Qt6::QuickControls2 Qt6::Test)
    add_test(NAME qt_zip_browser COMMAND xdzip-browser-tests -v1 -o "${CMAKE_CURRENT_BINARY_DIR}/browser-tests.log,txt")
    # Whole-suite ceiling includes large scans and many event-loop waits. Individual
    # QTRY assertions keep their own deadlines; a slow host must report the assertion.
    set_tests_properties(qt_zip_browser PROPERTIES TIMEOUT 120)
    add_test(NAME qt_browser_load COMMAND xdzip-browser --smoke-test)
endif()
foreach(target xdzip-browser xdzip-browser-tests)
    if(TARGET ${target})
        if(MSVC)
            target_compile_options(${target} PRIVATE /W4 /WX /utf-8)
        else()
            target_compile_options(${target} PRIVATE -Wall -Wextra -Werror)
        endif()
    endif()
endforeach()

foreach(target xdzip-ui-preview xdzip-ui-tests xdzip-browser xdzip-browser-tests)
    if(TARGET ${target})
        target_link_libraries(${target} PRIVATE xdzip-ui-resourcesplugin)
    endif()
endforeach()
