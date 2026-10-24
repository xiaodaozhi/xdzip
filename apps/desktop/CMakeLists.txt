set(XDZIP_MACOS_HANDLER_RANK "None" CACHE STRING "macOS document handling rank: None for development/test copies, Alternate for the primary application")
set_property(CACHE XDZIP_MACOS_HANDLER_RANK PROPERTY STRINGS None Alternate)
if(NOT XDZIP_MACOS_HANDLER_RANK MATCHES "^(None|Alternate)$")
    message(FATAL_ERROR "XDZIP_MACOS_HANDLER_RANK must be None or Alternate")
endif()

find_package(Qt6 6.8 REQUIRED COMPONENTS Core Gui Qml Quick QuickControls2)
qt_standard_project_setup()
qt_policy(SET QTP0001 NEW)
set(XDZIP_BUILD_INFO_DIR "${CMAKE_CURRENT_BINARY_DIR}/generated")
add_custom_target(xdzip-build-info
    COMMAND "${CMAKE_COMMAND}" "-DXDZIP_SOURCE_DIR=${PROJECT_SOURCE_DIR}"
        "-DXDZIP_OUTPUT_DIR=${XDZIP_BUILD_INFO_DIR}" "-DXDZIP_DISPLAY_VERSION=${XDZIP_DISPLAY_VERSION}"
        "-DXDZIP_BUILD_NUMBER=${XDZIP_BUILD_NUMBER}" "-DXDZIP_CHANNEL=${XDZIP_CHANNEL}"
        "-DXDZIP_CONFIGURATION=${CMAKE_BUILD_TYPE}"
        -P "${PROJECT_SOURCE_DIR}/cmake/BuildVersion.cmake"
    BYPRODUCTS "${XDZIP_BUILD_INFO_DIR}/BuildVersion.h" VERBATIM)
set_property(SOURCE BuildInfo.cpp APPEND PROPERTY OBJECT_DEPENDS
    "${XDZIP_BUILD_INFO_DIR}/BuildVersion.h")
qt_add_resources(ui_resources ui.qrc)
add_library(xdzip-ui-resources STATIC ${ui_resources})
add_dependencies(xdzip-ui-resources xdzip-build-info)
target_include_directories(xdzip-ui-resources PUBLIC "${XDZIP_BUILD_INFO_DIR}")
qt_add_qml_module(xdzip-ui-resources
    URI Xdzip.Settings VERSION 1.0
    OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/qml/Xdzip/Settings"
    SOURCES UiPreferences.h UiPreferences.cpp WindowsFrame.h WindowsFrame.cpp BuildInfo.h BuildInfo.cpp)
target_link_libraries(xdzip-ui-resources PRIVATE Qt6::Core Qt6::Gui Qt6::Qml)
if(WIN32)
    target_link_libraries(xdzip-ui-resources PRIVATE Qt6::GuiPrivate dwmapi user32)
endif()
add_executable(xdzip-ui-preview main.cpp)
target_link_libraries(xdzip-ui-preview PRIVATE Qt6::Quick Qt6::QuickControls2)
if(BUILD_TESTING)
    add_test(NAME qt_preview_load COMMAND xdzip-ui-preview --smoke-test)
    find_package(Qt6 6.8 REQUIRED COMPONENTS QuickTest)
    add_executable(xdzip-ui-tests tests/main.cpp)
    target_link_libraries(xdzip-ui-tests PRIVATE Qt6::QuickTest Qt6::QuickControls2)
    add_test(NAME qt_task_components COMMAND xdzip-ui-tests -input "${CMAKE_CURRENT_SOURCE_DIR}/tests" -o qml-tests.log,txt -o -,txt)
    set_tests_properties(qt_task_components PROPERTIES TIMEOUT 60)
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

find_package(Qt6 6.8 REQUIRED COMPONENTS Widgets Network)
include(${PROJECT_SOURCE_DIR}/cmake/RustStaticLibrary.cmake)
xdzip_rust_library(xdzip-reader-rust xdzip-qt-bridge xdzip_qt_bridge)
qt_add_executable(xdzip-browser browser_main.cpp)
# Keep the internal target stable while using the product name on every platform.
set_target_properties(xdzip-browser PROPERTIES OUTPUT_NAME xdzip)
# Linker evidence remains a local/CI artifact. It does not change optimization,
# safety checks or runtime behavior, and is not a complete license attribution.
if(APPLE)
    target_link_options(xdzip-browser PRIVATE "LINKER:-map,${CMAKE_CURRENT_BINARY_DIR}/xdzip-link.map")
elseif(MSVC)
    target_link_options(xdzip-browser PRIVATE "/MAP:${CMAKE_CURRENT_BINARY_DIR}/xdzip-link.map")
else()
    target_link_options(xdzip-browser PRIVATE "LINKER:-Map,${CMAKE_CURRENT_BINARY_DIR}/xdzip-link.map")
endif()

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
    include(${PROJECT_SOURCE_DIR}/cmake/Sparkle.cmake)
    set_target_properties(xdzip-browser PROPERTIES OBJCXX_STANDARD 17
        OBJCXX_STANDARD_REQUIRED ON OBJCXX_EXTENSIONS OFF)
    target_sources(xdzip-browser PRIVATE MacTitlebar.mm)
    target_sources(xdzip-browser PRIVATE MacUpdateProvider.mm MacUpdateProvider.h)
    set_source_files_properties(MacUpdateProvider.mm PROPERTIES COMPILE_OPTIONS "-fobjc-arc;-fblocks;-Werror=protocol")
    target_link_libraries(xdzip-browser PRIVATE "${XDZIP_SPARKLE_FRAMEWORK}")
    set_property(TARGET xdzip-browser APPEND PROPERTY BUILD_RPATH "${XDZIP_SPARKLE_SDK}")
    target_link_libraries(xdzip-browser PRIVATE "-framework AppKit")
    set_target_properties(xdzip-browser PROPERTIES MACOSX_BUNDLE TRUE
        MACOSX_BUNDLE_GUI_IDENTIFIER "org.xdzip.desktop"
        MACOSX_BUNDLE_INFO_PLIST "${CMAKE_CURRENT_SOURCE_DIR}/packaging/macos/Info.plist.in"
        MACOSX_BUNDLE_BUNDLE_NAME "xdzip"
        MACOSX_BUNDLE_ICON_FILE "xdzip.icns"
        MACOSX_BUNDLE_SHORT_VERSION_STRING "${PROJECT_VERSION}"
        MACOSX_BUNDLE_BUNDLE_VERSION "${XDZIP_BUILD_NUMBER}"
        MACOSX_BUNDLE_LONG_VERSION_STRING "${XDZIP_DISPLAY_VERSION}")
    set_source_files_properties("${XDZIP_BRAND_ASSETS}/xdzip.icns" PROPERTIES
        MACOSX_PACKAGE_LOCATION "Resources")
    target_sources(xdzip-browser PRIVATE "${XDZIP_BRAND_ASSETS}/xdzip.icns")
    set_source_files_properties("${XDZIP_BRAND_ASSETS}/xdzip-zip.icns" PROPERTIES MACOSX_PACKAGE_LOCATION "Resources")
    target_sources(xdzip-browser PRIVATE "${XDZIP_BRAND_ASSETS}/xdzip-zip.icns")
endif()
qt_add_qml_module(xdzip-browser
    URI Xdzip.Native VERSION 1.0
    DEPENDENCIES QtQml.Models
    OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/qml/Xdzip/Native"
    SOURCES BrowserController.h BrowserController.cpp RuntimeConnection.h RuntimeConnection.cpp DesktopIntegration.h DesktopIntegration.cpp ApplicationSessions.h ApplicationSessions.cpp SingleInstance.h SingleInstance.cpp UpdateController.h UpdateController.cpp UpdateHandoff.h UpdateHandoff.cpp)
target_link_libraries(xdzip-browser PRIVATE xdzip-reader-rust
    Qt6::Widgets Qt6::Network Qt6::Quick Qt6::QuickControls2 ${CMAKE_DL_LIBS})
include(${PROJECT_SOURCE_DIR}/cmake/UpdateHelper.cmake)
option(XDZIP_NATIVE_UPDATE_FIXTURE "Compile private native updater regression instrumentation" OFF)
option(XDZIP_NATIVE_MAC_UPDATE_FIXTURE "Compile private macOS product updater regression instrumentation" OFF)
if(XDZIP_NATIVE_UPDATE_FIXTURE)
    if(APPLE OR XDZIP_NATIVE_MAC_UPDATE_FIXTURE OR
       NOT "$ENV{XDZIP_ISOLATED_UPDATE_FIXTURE}" STREQUAL "1" OR
       NOT IS_ABSOLUTE "${XDZIP_NATIVE_UPDATE_FIXTURE_DIRECTORY}")
        message(FATAL_ERROR "Native update fixture requires an explicit private Windows/Linux regression build")
    endif()
    find_package(Qt6 6.8 REQUIRED COMPONENTS Test)
    target_link_libraries(xdzip-browser PRIVATE Qt6::Test)
    target_sources(xdzip-browser PRIVATE tests/native_update_fixture.cpp tests/native_update_fixture.h)
    target_include_directories(xdzip-browser PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
    target_compile_definitions(xdzip-browser PRIVATE XDZIP_NATIVE_UPDATE_FIXTURE
        "XDZIP_NATIVE_UPDATE_FIXTURE_DIRECTORY=\"${XDZIP_NATIVE_UPDATE_FIXTURE_DIRECTORY}\"")
endif()
if(XDZIP_NATIVE_MAC_UPDATE_FIXTURE)
    if(NOT APPLE OR XDZIP_NATIVE_UPDATE_FIXTURE OR
       NOT "$ENV{XDZIP_ISOLATED_UPDATE_FIXTURE}" STREQUAL "1" OR
       NOT IS_ABSOLUTE "${XDZIP_NATIVE_MAC_UPDATE_FIXTURE_DIRECTORY}")
        message(FATAL_ERROR "macOS product update fixture requires an explicit private native regression build")
    endif()
    find_package(Qt6 6.8 REQUIRED COMPONENTS Test)
    target_link_libraries(xdzip-browser PRIVATE Qt6::Test)
    target_sources(xdzip-browser PRIVATE tests/mac_product_update_fixture.mm tests/mac_product_update_fixture.h)
    target_include_directories(xdzip-browser PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
    set_source_files_properties(tests/mac_product_update_fixture.mm PROPERTIES COMPILE_OPTIONS "-fobjc-arc;-fblocks")
    target_compile_definitions(xdzip-browser PRIVATE XDZIP_NATIVE_MAC_UPDATE_FIXTURE
        "XDZIP_NATIVE_MAC_UPDATE_FIXTURE_DIRECTORY=\"${XDZIP_NATIVE_MAC_UPDATE_FIXTURE_DIRECTORY}\"")
endif()
if(UNIX AND NOT APPLE)
    include(GNUInstallDirs)
    install(TARGETS xdzip-browser RUNTIME DESTINATION "${CMAKE_INSTALL_BINDIR}")
    install(FILES packaging/linux/xdzip.desktop DESTINATION "${CMAKE_INSTALL_DATADIR}/applications")
    install(FILES "${XDZIP_BRAND_ASSETS}/xdzip.png"
        DESTINATION "${CMAKE_INSTALL_DATADIR}/icons/hicolor/512x512/apps")
endif()
if(BUILD_TESTING)
    add_executable(xdzip-browser-tests tests/browser_test.cpp BrowserController.cpp BrowserController.h RuntimeConnection.cpp RuntimeConnection.h DesktopIntegration.h DesktopIntegration.cpp ApplicationSessions.h ApplicationSessions.cpp SingleInstance.h SingleInstance.cpp)
    if(APPLE)
        target_sources(xdzip-browser-tests PRIVATE MacTitlebar.mm tests/native_dialog_mac.mm)
        target_link_libraries(xdzip-browser-tests PRIVATE "-framework AppKit")
        set_target_properties(xdzip-browser-tests PROPERTIES OBJCXX_STANDARD 17 OBJCXX_STANDARD_REQUIRED ON)
    endif()
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
        Qt6::Widgets Qt6::Network Qt6::Quick Qt6::QuickControls2 Qt6::Test)
    add_test(NAME qt_zip_browser COMMAND xdzip-browser-tests -v1 -o "${CMAKE_CURRENT_BINARY_DIR}/browser-tests.log,txt")
    # Whole-suite ceiling includes large scans and many event-loop waits. Individual
    # QTRY assertions keep their own deadlines; a slow host must report the assertion.
    # The session, resizable-list and About-state matrices run together here.
    # Their per-assertion and separate cold-menu budgets remain unchanged.
    set_tests_properties(qt_zip_browser PROPERTIES TIMEOUT 240)
    add_test(NAME qt_browser_load COMMAND xdzip-browser --smoke-test)
    add_executable(xdzip-update-tests tests/update_controller_test.cpp
        UpdateController.cpp UpdateController.h UpdateHandoff.cpp UpdateHandoff.h
        BrowserController.cpp BrowserController.h RuntimeConnection.cpp RuntimeConnection.h
        DesktopIntegration.cpp DesktopIntegration.h ApplicationSessions.cpp ApplicationSessions.h
        SingleInstance.cpp SingleInstance.h)
    target_include_directories(xdzip-update-tests PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
    target_compile_definitions(xdzip-update-tests PRIVATE XDZIP_UPDATE_FIXTURES="${PROJECT_SOURCE_DIR}/tests/fixtures/update")
    target_link_libraries(xdzip-update-tests PRIVATE xdzip-reader-rust xdzip-ui-resourcesplugin
        Qt6::Widgets Qt6::Network Qt6::Quick Qt6::QuickControls2 Qt6::Test)
    if(APPLE)
        target_sources(xdzip-update-tests PRIVATE MacTitlebar.mm MacUpdateProvider.mm)
        target_compile_definitions(xdzip-update-tests PRIVATE XDZIP_UPDATE_TESTS)
        target_link_libraries(xdzip-update-tests PRIVATE "-framework AppKit" "${XDZIP_SPARKLE_FRAMEWORK}")
        set_property(TARGET xdzip-update-tests APPEND PROPERTY BUILD_RPATH "${XDZIP_SPARKLE_SDK}")
        set_target_properties(xdzip-update-tests PROPERTIES OBJCXX_STANDARD 17 OBJCXX_STANDARD_REQUIRED ON)
        option(XDZIP_BUILD_MAC_UPDATE_REGRESSION "Build the isolated native Mac update probe" OFF)
        if(XDZIP_BUILD_MAC_UPDATE_REGRESSION)
            set(XDZIP_MAC_UPDATE_PROBE_BUILD "730" CACHE STRING "Owned native updater fixture build")
            if(NOT XDZIP_MAC_UPDATE_PROBE_BUILD MATCHES "^[0-9]+$")
                message(FATAL_ERROR "Invalid Mac updater fixture build")
            endif()
            get_target_property(probe_sources xdzip-update-tests SOURCES)
            list(FILTER probe_sources EXCLUDE REGEX "tests/update_controller_test\\.cpp$")
            add_executable(xdzip-mac-update-probe MACOSX_BUNDLE tests/mac_update_probe.mm ${probe_sources})
            target_include_directories(xdzip-mac-update-probe PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
            target_link_libraries(xdzip-mac-update-probe PRIVATE xdzip-reader-rust xdzip-ui-resourcesplugin
                Qt6::Widgets Qt6::Network Qt6::Quick Qt6::QuickControls2 Qt6::Test
                "-framework AppKit" "${XDZIP_SPARKLE_FRAMEWORK}")
            target_compile_options(xdzip-mac-update-probe PRIVATE -Wall -Wextra -Werror)
            set_property(TARGET xdzip-mac-update-probe APPEND PROPERTY BUILD_RPATH "${XDZIP_SPARKLE_SDK}")
            set_source_files_properties(tests/mac_update_probe.mm PROPERTIES COMPILE_OPTIONS "-fobjc-arc;-fblocks")
            set_target_properties(xdzip-mac-update-probe PROPERTIES
                OBJCXX_STANDARD 17 OBJCXX_STANDARD_REQUIRED ON OUTPUT_NAME "xdzip-update-probe"
                MACOSX_BUNDLE_GUI_IDENTIFIER "org.xdzip.update-regression"
                MACOSX_BUNDLE_BUNDLE_NAME "xdzip update regression"
                MACOSX_BUNDLE_BUNDLE_VERSION "${XDZIP_MAC_UPDATE_PROBE_BUILD}"
                MACOSX_BUNDLE_SHORT_VERSION_STRING "99.0.0"
                MACOSX_BUNDLE_INFO_PLIST "${CMAKE_CURRENT_SOURCE_DIR}/packaging/macos/Info.plist.in")
        endif()
    endif()
    add_test(NAME qt_updates COMMAND xdzip-update-tests -v1 -o "${CMAKE_CURRENT_BINARY_DIR}/update-tests.log,txt")
    set_tests_properties(qt_updates PROPERTIES TIMEOUT 60)
endif()
foreach(target xdzip-browser xdzip-browser-tests xdzip-update-tests)
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

# External file transfers share the controller and immutable private-copy lifetime.
foreach(transfer_target xdzip-browser xdzip-browser-tests xdzip-update-tests)
    if(TARGET ${transfer_target})
        target_sources(${transfer_target} PRIVATE BrowserTransfers.cpp NativeExport.cpp)
        if(WIN32)
            target_sources(${transfer_target} PRIVATE NativeExportWindows.cpp)
            target_link_libraries(${transfer_target} PRIVATE ole32 shell32 shlwapi)
        elseif(APPLE)
            target_sources(${transfer_target} PRIVATE NativeExportMac.mm)
            set_source_files_properties(NativeExportMac.mm PROPERTIES COMPILE_OPTIONS "-fobjc-arc;-fblocks;-Werror=protocol")
            target_link_libraries(${transfer_target} PRIVATE "-framework AppKit")
        endif()
    endif()
endforeach()
