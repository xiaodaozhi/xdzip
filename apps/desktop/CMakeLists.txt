set(XDZIP_MACOS_HANDLER_RANK "None" CACHE STRING "macOS document handling rank: None for development/test copies, Alternate for the primary application")
set_property(CACHE XDZIP_MACOS_HANDLER_RANK PROPERTY STRINGS None Alternate)
if(NOT XDZIP_MACOS_HANDLER_RANK MATCHES "^(None|Alternate)$")
    message(FATAL_ERROR "XDZIP_MACOS_HANDLER_RANK must be None or Alternate")
endif()
set(XDZIP_MACOS_DOCUMENT_ICONS zip tar tar-gz gz 7z rar cab archive iso udf)

find_package(Qt6 6.8 REQUIRED COMPONENTS Core Gui Qml Quick QuickControls2)
find_package(Python3 3.10 REQUIRED COMPONENTS Interpreter)
get_filename_component(XDZIP_QT_SDK_PREFIX "${Qt6_DIR}/../../.." ABSOLUTE)
execute_process(COMMAND "${Python3_EXECUTABLE}" "${PROJECT_SOURCE_DIR}/scripts/prepare-qt-network.py"
    --qt-prefix "${XDZIP_QT_SDK_PREFIX}" --check-only
    RESULT_VARIABLE XDZIP_QT_NETWORK_SECURITY_STATUS OUTPUT_QUIET ERROR_QUIET)
if(NOT XDZIP_QT_NETWORK_SECURITY_STATUS EQUAL 0)
    message(FATAL_ERROR "Prepare the pinned QtNetwork security build with scripts/prepare-qt-network.py before configuring the desktop")
endif()
qt_standard_project_setup()
qt_policy(SET QTP0001 NEW)
qt_policy(SET QTP0004 NEW)
add_library(xdzip-password-accessibility STATIC PasswordAccessibility.cpp PasswordAccessibility.h)
target_link_libraries(xdzip-password-accessibility PRIVATE Qt6::QuickPrivate)
set(XDZIP_BUILD_INFO_DIR "${CMAKE_CURRENT_BINARY_DIR}/generated")
set(XDZIP_LANGUAGE_REGISTRY "${PROJECT_SOURCE_DIR}/resources/ui-languages.json")
set(XDZIP_LANGUAGE_GENERATOR "${PROJECT_SOURCE_DIR}/scripts/ui_languages.py")
set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS
    "${XDZIP_LANGUAGE_REGISTRY}" "${XDZIP_LANGUAGE_GENERATOR}")
execute_process(COMMAND "${Python3_EXECUTABLE}" -B "${XDZIP_LANGUAGE_GENERATOR}"
    --cpp "${XDZIP_BUILD_INFO_DIR}/UiLanguageData.h" COMMAND_ERROR_IS_FATAL ANY)
include_directories("${XDZIP_BUILD_INFO_DIR}")
add_custom_target(xdzip-build-info
    COMMAND "${CMAKE_COMMAND}" "-DXDZIP_SOURCE_DIR=${PROJECT_SOURCE_DIR}"
        "-DXDZIP_IDENTITY_PYTHON=${Python3_EXECUTABLE}"
        "-DXDZIP_OUTPUT_DIR=${XDZIP_BUILD_INFO_DIR}" "-DXDZIP_DISPLAY_VERSION=${XDZIP_DISPLAY_VERSION}"
        "-DXDZIP_BUILD_NUMBER=${XDZIP_BUILD_NUMBER}" "-DXDZIP_CHANNEL=${XDZIP_CHANNEL}"
        "-DXDZIP_CONFIGURATION=${CMAKE_BUILD_TYPE}"
        -P "${PROJECT_SOURCE_DIR}/cmake/BuildVersion.cmake"
    BYPRODUCTS "${XDZIP_BUILD_INFO_DIR}/BuildVersion.h" VERBATIM)
set_property(SOURCE BuildInfo.cpp APPEND PROPERTY OBJECT_DEPENDS
    "${XDZIP_BUILD_INFO_DIR}/BuildVersion.h")
set(XDZIP_FONT_DIR "${PROJECT_BINARY_DIR}/generated/fonts")
set(XDZIP_FONT_TTF "${XDZIP_FONT_DIR}/xdzip-design-sans-sc.ttf")
file(GLOB XDZIP_LANGUAGE_CATALOGS CONFIGURE_DEPENDS "${PROJECT_SOURCE_DIR}/resources/i18n/*.json")
set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS
    "${PROJECT_SOURCE_DIR}/scripts/ui_fonts.py" "${PROJECT_SOURCE_DIR}/specs/dependencies/ui-font.json")
execute_process(COMMAND "${Python3_EXECUTABLE}" -B "${PROJECT_SOURCE_DIR}/scripts/ui-fonts.py"
    localized-files OUTPUT_VARIABLE XDZIP_LOCALIZED_FONT_NAMES OUTPUT_STRIP_TRAILING_WHITESPACE
    COMMAND_ERROR_IS_FATAL ANY)
set(XDZIP_LOCALIZED_FONT_FILES "")
set(XDZIP_LOCALIZED_FONT_RESOURCES "")
foreach(font IN LISTS XDZIP_LOCALIZED_FONT_NAMES)
    list(APPEND XDZIP_LOCALIZED_FONT_FILES "${XDZIP_FONT_DIR}/${font}")
    string(APPEND XDZIP_LOCALIZED_FONT_RESOURCES "<file alias=\"fonts/${font}\">${XDZIP_FONT_DIR}/${font}</file>\n")
endforeach()
# Rescan on every build so newly added/deleted text files are observed too.
# The generator preserves unchanged output bytes and timestamps.
add_custom_target(xdzip-ui-font
    COMMAND Python3::Interpreter "${PROJECT_SOURCE_DIR}/scripts/ui-fonts.py" build --output-dir "${XDZIP_FONT_DIR}"
    BYPRODUCTS "${XDZIP_FONT_TTF}" ${XDZIP_LOCALIZED_FONT_FILES}
        "${XDZIP_FONT_DIR}/xdzip-design-sans-sc.woff2" "${XDZIP_FONT_DIR}/manifest.json"
    VERBATIM)
set_source_files_properties("${XDZIP_FONT_TTF}" ${XDZIP_LOCALIZED_FONT_FILES} PROPERTIES GENERATED TRUE)
# Preserve the existing ui resource initialization and aliases. Source asset
# paths become absolute because this combined qrc lives in the build tree.
file(READ "${CMAKE_CURRENT_SOURCE_DIR}/ui.qrc" XDZIP_UI_QRC)
string(REGEX REPLACE "<file>([^<]+)</file>" "<file alias=\"\\1\">\\1</file>" XDZIP_UI_QRC "${XDZIP_UI_QRC}")
string(REGEX REPLACE "(<file[^>]*>)([^<]+)" "\\1${CMAKE_CURRENT_SOURCE_DIR}/\\2" XDZIP_UI_QRC "${XDZIP_UI_QRC}")
set(XDZIP_I18N_RESOURCES "")
foreach(catalog IN LISTS XDZIP_LANGUAGE_CATALOGS)
    get_filename_component(catalog_name "${catalog}" NAME)
    string(APPEND XDZIP_I18N_RESOURCES "<file alias=\"i18n/${catalog_name}\">${catalog}</file>\n")
endforeach()
set(XDZIP_PACKAGING_GENERATOR "${PROJECT_SOURCE_DIR}/scripts/packaging_languages.py")
set(XDZIP_PACKAGING_TRANSLATIONS "${XDZIP_BUILD_INFO_DIR}/packaging-localizations")
set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS
    "${XDZIP_PACKAGING_GENERATOR}" "${PROJECT_SOURCE_DIR}/resources/packaging-i18n.json")
string(REPLACE "</RCC>" "  <qresource prefix=\"/xdzip\">${XDZIP_I18N_RESOURCES}<file alias=\"fonts/xdzip-design-sans-sc.ttf\">${XDZIP_FONT_TTF}</file>${XDZIP_LOCALIZED_FONT_RESOURCES}</qresource>\n</RCC>" XDZIP_UI_QRC "${XDZIP_UI_QRC}")
file(CONFIGURE OUTPUT "${CMAKE_CURRENT_BINARY_DIR}/ui.qrc" CONTENT "${XDZIP_UI_QRC}" @ONLY)
qt_add_resources(ui_resources "${CMAKE_CURRENT_BINARY_DIR}/ui.qrc")
add_library(xdzip-ui-resources STATIC ${ui_resources})
add_dependencies(xdzip-ui-resources xdzip-ui-font)
add_dependencies(xdzip-ui-resources xdzip-image-preview)
add_dependencies(xdzip-ui-resources xdzip-build-info)
qt_add_qml_module(xdzip-ui-resources
    URI Xdzip.Settings VERSION 1.0
    DEPENDENCIES QtQuick
    OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/qml/Xdzip/Settings"
    SOURCES PreviewImages.h PreviewImages.cpp ImagePreviewProcess.h ImagePreviewProcess.cpp UiPreferences.h UiPreferences.cpp UiFonts.h WindowsFrame.h WindowsFrame.cpp LinuxFrame.h LinuxFrame.cpp BuildInfo.h BuildInfo.cpp)
# Compile the QML declared in the resource manifest ahead of first launch.
# Keep the original source resources available for Qt's cache fallback.
set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS "${CMAKE_CURRENT_SOURCE_DIR}/ui.qrc")
file(STRINGS "${CMAKE_CURRENT_SOURCE_DIR}/ui.qrc" XDZIP_QML_FILES
    REGEX "<file>ui/[^<]+\\.qml</file>")
list(TRANSFORM XDZIP_QML_FILES REPLACE "^[ \t]*<file>([^<]+)</file>.*" "\\1")
if(NOT XDZIP_QML_FILES)
    message(FATAL_ERROR "No QML files found in ui.qrc")
endif()
set_target_properties(xdzip-ui-resources PROPERTIES QT_QMLCACHEGEN_ARGUMENTS "--only-bytecode")
qt_target_qml_sources(xdzip-ui-resources NO_QMLDIR_TYPES PREFIX "/xdzip" QML_FILES ${XDZIP_QML_FILES})
target_link_libraries(xdzip-ui-resources PRIVATE Qt6::Core Qt6::Gui Qt6::Qml Qt6::Quick)
if(WIN32)
    target_link_libraries(xdzip-ui-resources PRIVATE dwmapi user32)
elseif(CMAKE_SYSTEM_NAME STREQUAL "Linux")
    find_package(X11 REQUIRED COMPONENTS Xext)
    target_link_libraries(xdzip-ui-resources PRIVATE X11::X11 X11::Xext)
endif()
add_executable(xdzip-ui-preview main.cpp)
target_link_libraries(xdzip-ui-preview PRIVATE Qt6::Quick Qt6::QuickControls2)
if(BUILD_TESTING)
    if(CMAKE_SYSTEM_NAME STREQUAL "Linux")
        add_executable(xdzip-private-cache-test tests/private_cache_test.cpp PrivateCache.cpp PrivateCache.h)
        target_include_directories(xdzip-private-cache-test PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
        target_link_libraries(xdzip-private-cache-test PRIVATE Qt6::Core)
        add_test(NAME qt_private_cache COMMAND xdzip-private-cache-test)
        add_executable(xdzip-x11-drag-gesture-test tests/x11_drag_gesture_test.cpp NativeDragGesture.cpp NativeDragGesture.h)
        target_include_directories(xdzip-x11-drag-gesture-test PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
        target_link_libraries(xdzip-x11-drag-gesture-test PRIVATE Qt6::Gui X11::X11 ${CMAKE_DL_LIBS})
        find_program(XDZIP_XVFB_RUN xvfb-run)
        find_library(XDZIP_XTEST_RUNTIME NAMES Xtst libXtst.so.6)
        if(XDZIP_XVFB_RUN AND XDZIP_XTEST_RUNTIME)
            add_test(NAME qt_x11_drag_gesture COMMAND "${XDZIP_XVFB_RUN}" -a -s "-screen 0 1024x768x24 -screen 1 640x480x24" $<TARGET_FILE:xdzip-x11-drag-gesture-test>)
            set_tests_properties(qt_x11_drag_gesture PROPERTIES ENVIRONMENT "QT_QPA_PLATFORM=xcb")
            add_executable(xdzip-x11-drag-handoff-test tests/x11_drag_handoff_test.cpp NativeDragGesture.cpp NativeDragGesture.h)
            target_include_directories(xdzip-x11-drag-handoff-test PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
            target_link_libraries(xdzip-x11-drag-handoff-test PRIVATE Qt6::Gui X11::X11 ${CMAKE_DL_LIBS})
            add_test(NAME qt_x11_drag_handoff COMMAND "${XDZIP_XVFB_RUN}" -a -s "-screen 0 1024x768x24" $<TARGET_FILE:xdzip-x11-drag-handoff-test>)
            set_tests_properties(qt_x11_drag_handoff PROPERTIES ENVIRONMENT "QT_QPA_PLATFORM=xcb" TIMEOUT 20)
        endif()
    endif()
    add_test(NAME qt_preview_load COMMAND xdzip-ui-preview --smoke-test)
    find_package(Qt6 6.8 REQUIRED COMPONENTS QuickTest)
    add_executable(xdzip-ui-tests tests/main.cpp tests/test_environment.cpp tests/test_environment.h)
    target_compile_definitions(xdzip-ui-tests PRIVATE XDZIP_TEST_TEMP_ROOT="${PROJECT_SOURCE_DIR}/build/test-tmp")
    target_link_libraries(xdzip-ui-tests PRIVATE Qt6::QuickTest Qt6::QuickControls2)
    add_test(NAME qt_task_components COMMAND xdzip-ui-tests -input "${CMAKE_CURRENT_SOURCE_DIR}/tests" -o qml-tests.log,txt -o -,txt)
    set_tests_properties(qt_task_components PROPERTIES TIMEOUT 60)
endif()

foreach(target xdzip-password-accessibility xdzip-ui-resources xdzip-ui-preview xdzip-ui-tests)
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
add_library(xdzip-desktop-session STATIC)
qt_add_qml_module(xdzip-desktop-session
    URI Xdzip.Native VERSION 1.0
    DEPENDENCIES QtQml.Models
    OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/qml/Xdzip/Native"
    SOURCES BrowserController.h BrowserController.cpp BrowserTransfers.cpp
        RuntimeConnection.h RuntimeConnection.cpp DesktopIntegration.h DesktopIntegration.cpp
        ApplicationSessions.h ApplicationSessions.cpp SingleInstance.h SingleInstance.cpp
        ShellBatchController.h ShellBatchController.cpp ShellActionRequest.h ShellActionRequest.cpp
        NativeExport.h NativeExport.cpp NativeDragGesture.h NativeDragGesture.cpp
        PrivateCache.h PrivateCache.cpp)
target_link_libraries(xdzip-desktop-session
    PUBLIC xdzip-reader-rust Qt6::Widgets Qt6::Network Qt6::Quick
    PRIVATE xdzip-ui-resources ${CMAKE_DL_LIBS})
if(WIN32)
    target_sources(xdzip-desktop-session PRIVATE NativeExportWindows.cpp)
    target_link_libraries(xdzip-desktop-session PRIVATE ole32 shell32 shlwapi)
elseif(APPLE)
    enable_language(OBJCXX)
    target_sources(xdzip-desktop-session PRIVATE MacTitlebar.mm MacWindowActivation.mm MacQuitMenu.mm NativeExportMac.mm)
    set_source_files_properties(MacQuitMenu.mm PROPERTIES COMPILE_OPTIONS "-fobjc-arc")
    set_source_files_properties(NativeExportMac.mm PROPERTIES COMPILE_OPTIONS "-fobjc-arc;-fblocks;-Werror=protocol")
    set_target_properties(xdzip-desktop-session PROPERTIES OBJCXX_STANDARD 17
        OBJCXX_STANDARD_REQUIRED ON OBJCXX_EXTENSIONS OFF)
    target_link_libraries(xdzip-desktop-session PRIVATE "-framework AppKit" "-framework Security")
elseif(CMAKE_SYSTEM_NAME STREQUAL "Linux")
    target_link_libraries(xdzip-desktop-session PRIVATE X11::X11)
endif()

set(xdzip_update_sources CliRequestBroker.h CliRequestBroker.cpp UpdateController.h UpdateController.cpp
    UpdateHandoff.h UpdateHandoff.cpp)
qt_add_executable(xdzip-browser browser_main.cpp ShellIntegration.h ShellIntegration.cpp)
target_sources(xdzip-browser PRIVATE ${xdzip_update_sources} FileAssociations.h FileAssociations.cpp)
target_link_libraries(xdzip-browser PRIVATE xdzip-desktop-sessionplugin xdzip-desktop-session Qt6::QuickControls2)
if(WIN32)
    target_link_libraries(xdzip-browser PRIVATE windowsapp)
endif()
add_dependencies(xdzip-browser xdzip-image-preview)
target_sources(xdzip-browser PRIVATE WindowsAssociationDescriptions.cpp WindowsAssociationDescriptions.h)
if(WIN32)
    target_link_libraries(xdzip-browser PRIVATE shell32 shlwapi)
    add_executable(xdzip-association-launcher WIN32 AssociationLauncher.cpp)
    set_target_properties(xdzip-association-launcher PROPERTIES OUTPUT_NAME xdzip-association)
    target_link_libraries(xdzip-association-launcher PRIVATE shell32 ole32)
    add_dependencies(xdzip-browser xdzip-association-launcher)
    if(MSVC)
        target_compile_options(xdzip-association-launcher PRIVATE /W4 /WX /utf-8)
    endif()
endif()
# Keep the internal target stable while using the product name on every platform.
set_target_properties(xdzip-browser PROPERTIES OUTPUT_NAME xdzip)
xdzip_rust_profile(XDZIP_CLI_PROFILE)
set(XDZIP_CLI_BUILD "${CMAKE_BINARY_DIR}/cli")
set(XDZIP_CLI_BUNDLE "${CMAKE_CURRENT_BINARY_DIR}/cli")
if(APPLE)
    set(XDZIP_APP_BUNDLE "${CMAKE_CURRENT_BINARY_DIR}/xdzip.app/Contents")
    set(XDZIP_CLI_BUNDLE "${XDZIP_APP_BUNDLE}/Helpers/cli")
endif()
add_custom_target(xdzip-command
    COMMAND Python3::Interpreter "${PROJECT_SOURCE_DIR}/scripts/cli.py" build
        --build-dir "${XDZIP_CLI_BUILD}" --profile "${XDZIP_CLI_PROFILE}"
        --codec "$<TARGET_FILE:xdzip-native-codec>"
    COMMAND Python3::Interpreter "${PROJECT_SOURCE_DIR}/scripts/cli.py" bundle
        --build-dir "${XDZIP_CLI_BUILD}" --output "${XDZIP_CLI_BUNDLE}"
    DEPENDS xdzip-native-codec
    WORKING_DIRECTORY "${PROJECT_SOURCE_DIR}" VERBATIM)
add_dependencies(xdzip-browser xdzip-command)

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
    set(XDZIP_RESOURCE_DESCRIPTION "xdzip Archive Manager")
    set(XDZIP_RESOURCE_FILENAME "xdzip.exe")
    set(XDZIP_RESOURCE_SHELL_PROTOCOL "1")
    configure_file(packaging/windows/app.rc.in "${CMAKE_CURRENT_BINARY_DIR}/xdzip.rc" @ONLY)
    target_sources(xdzip-browser PRIVATE "${CMAKE_CURRENT_BINARY_DIR}/xdzip.rc")
    set(XDZIP_RESOURCE_DESCRIPTION "xdzip")
    set(XDZIP_RESOURCE_FILENAME "xdzip-association.exe")
    set(XDZIP_RESOURCE_SHELL_PROTOCOL "0")
    configure_file(packaging/windows/app.rc.in "${CMAKE_CURRENT_BINARY_DIR}/xdzip-association.rc" @ONLY)
    target_sources(xdzip-association-launcher PRIVATE "${CMAKE_CURRENT_BINARY_DIR}/xdzip-association.rc")
endif()
if(APPLE)
    set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS ${XDZIP_LANGUAGE_CATALOGS}
        "${CMAKE_CURRENT_SOURCE_DIR}/packaging/macos/Info.plist.in")
    execute_process(COMMAND "${Python3_EXECUTABLE}" -B "${XDZIP_PACKAGING_GENERATOR}"
        --platform macos --output "${XDZIP_PACKAGING_TRANSLATIONS}"
        OUTPUT_VARIABLE XDZIP_MACOS_LANGUAGES OUTPUT_STRIP_TRAILING_WHITESPACE
        COMMAND_ERROR_IS_FATAL ANY)
    set(XDZIP_LOCALIZATION_XML "")
    foreach(locale IN LISTS XDZIP_MACOS_LANGUAGES)
        string(APPEND XDZIP_LOCALIZATION_XML "<string>${locale}</string>")
    endforeach()
    set(XDZIP_MACOS_BUNDLE_PLIST "${CMAKE_CURRENT_BINARY_DIR}/Localized-Info.plist.in")
    file(READ "${CMAKE_CURRENT_SOURCE_DIR}/packaging/macos/Info.plist.in" XDZIP_MACOS_PLIST)
    string(REPLACE "<array><string>zh-Hans</string><string>en</string></array>"
        "<array>${XDZIP_LOCALIZATION_XML}</array>" XDZIP_MACOS_PLIST "${XDZIP_MACOS_PLIST}")
    file(WRITE "${XDZIP_MACOS_BUNDLE_PLIST}" "${XDZIP_MACOS_PLIST}")
    include(${PROJECT_SOURCE_DIR}/cmake/Sparkle.cmake)
    set_target_properties(xdzip-browser PROPERTIES OBJCXX_STANDARD 17
        OBJCXX_STANDARD_REQUIRED ON OBJCXX_EXTENSIONS OFF)
    target_sources(xdzip-browser PRIVATE MacFileAssociations.mm)
    set_source_files_properties(MacFileAssociations.mm PROPERTIES COMPILE_OPTIONS "-fobjc-arc;-fblocks")
    target_link_libraries(xdzip-browser PRIVATE "-framework UniformTypeIdentifiers")
    target_sources(xdzip-browser PRIVATE MacUpdateProvider.mm MacUpdateProvider.h)
    set_source_files_properties(MacUpdateProvider.mm PROPERTIES COMPILE_OPTIONS "-fobjc-arc;-fblocks;-Werror=protocol")
    target_link_libraries(xdzip-browser PRIVATE "${XDZIP_SPARKLE_FRAMEWORK}")
    set_property(TARGET xdzip-browser APPEND PROPERTY BUILD_RPATH "${XDZIP_SPARKLE_SDK}")
    target_link_libraries(xdzip-browser PRIVATE "-framework AppKit")
    set_target_properties(xdzip-browser PROPERTIES MACOSX_BUNDLE TRUE
        MACOSX_BUNDLE_GUI_IDENTIFIER "org.xdzip.desktop"
        MACOSX_BUNDLE_INFO_PLIST "${XDZIP_MACOS_BUNDLE_PLIST}"
        MACOSX_BUNDLE_BUNDLE_NAME "xdzip"
        MACOSX_BUNDLE_COPYRIGHT "Copyright (C) 2026 xiaodaozhi"
        MACOSX_BUNDLE_ICON_FILE "xdzip.icns"
        MACOSX_BUNDLE_SHORT_VERSION_STRING "${PROJECT_VERSION}"
        MACOSX_BUNDLE_BUNDLE_VERSION "${XDZIP_BUILD_NUMBER}"
        MACOSX_BUNDLE_LONG_VERSION_STRING "${XDZIP_DISPLAY_VERSION}")
    set_source_files_properties("${XDZIP_BRAND_ASSETS}/xdzip.icns" PROPERTIES
        MACOSX_PACKAGE_LOCATION "Resources")
    target_sources(xdzip-browser PRIVATE "${XDZIP_BRAND_ASSETS}/xdzip.icns")
    foreach(icon IN LISTS XDZIP_MACOS_DOCUMENT_ICONS)
        set(document_icon "${PROJECT_SOURCE_DIR}/design/assets/file-associations/xdzip-${icon}.icns")
        set_source_files_properties("${document_icon}" PROPERTIES MACOSX_PACKAGE_LOCATION "Resources")
        target_sources(xdzip-browser PRIVATE "${document_icon}")
    endforeach()
endif()
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
    set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS
        "${CMAKE_CURRENT_SOURCE_DIR}/packaging/linux/xdzip.desktop"
        "${CMAKE_CURRENT_SOURCE_DIR}/packaging/linux/xdzip-mime.xml"
        "${CMAKE_CURRENT_SOURCE_DIR}/packaging/linux/xdzip-local.yml"
        "${CMAKE_CURRENT_SOURCE_DIR}/packaging/linux/org.xdzip.update.policy")
    execute_process(COMMAND "${Python3_EXECUTABLE}" -B "${XDZIP_PACKAGING_GENERATOR}"
        --platform linux --output "${XDZIP_PACKAGING_TRANSLATIONS}" COMMAND_ERROR_IS_FATAL ANY)
    install(FILES "${XDZIP_PACKAGING_TRANSLATIONS}/xdzip.desktop" DESTINATION "${CMAKE_INSTALL_DATADIR}/applications")
    install(FILES "${XDZIP_PACKAGING_TRANSLATIONS}/xdzip-mime.xml" DESTINATION "${CMAKE_INSTALL_DATADIR}/mime/packages")
    install(FILES "${XDZIP_BRAND_ASSETS}/xdzip.png"
        DESTINATION "${CMAKE_INSTALL_DATADIR}/icons/hicolor/512x512/apps")
endif()
if(BUILD_TESTING)
    add_executable(xdzip-association-description-tests tests/association_descriptions_test.cpp
        WindowsAssociationDescriptions.cpp WindowsAssociationDescriptions.h)
    target_include_directories(xdzip-association-description-tests PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
    find_package(Qt6 6.8 REQUIRED COMPONENTS Test)
    add_executable(xdzip-settings-placement-tests tests/settings_placement_test.cpp
        tests/test_environment.cpp tests/test_environment.h)
    target_include_directories(xdzip-settings-placement-tests PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
    target_compile_definitions(xdzip-settings-placement-tests PRIVATE XDZIP_TEST_TEMP_ROOT="${PROJECT_SOURCE_DIR}/build/test-tmp")
    target_link_libraries(xdzip-settings-placement-tests PRIVATE xdzip-ui-resourcesplugin
        Qt6::Widgets Qt6::Quick Qt6::QuickControls2 Qt6::Test)
    add_test(NAME qt_settings_placement COMMAND xdzip-settings-placement-tests)
    set_tests_properties(qt_settings_placement PROPERTIES TIMEOUT 90)
    target_link_libraries(xdzip-association-description-tests PRIVATE Qt6::Core Qt6::Test)
    add_executable(xdzip-image-fault tests/image_preview_fault.cpp)
    target_include_directories(xdzip-image-fault PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
    target_link_libraries(xdzip-image-fault PRIVATE Qt6::Core)
    add_executable(xdzip-image-tests tests/image_preview_test.cpp ImagePreviewProcess.cpp)
    target_include_directories(xdzip-image-tests PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
    target_link_libraries(xdzip-image-tests PRIVATE Qt6::Gui Qt6::Test)
    target_compile_definitions(xdzip-image-tests PRIVATE
        XDZIP_IMAGE_FIXTURES="${PROJECT_SOURCE_DIR}/tests/fixtures/preview"
        XDZIP_IMAGE_WORKER="$<TARGET_FILE:xdzip-image-preview>"
        XDZIP_IMAGE_FAULT_WORKER="$<TARGET_FILE:xdzip-image-fault>")
    add_dependencies(xdzip-image-tests xdzip-image-preview xdzip-image-fault)
    add_test(NAME qt_image_preview COMMAND xdzip-image-tests
        -o "${CMAKE_CURRENT_BINARY_DIR}/image-preview-tests.log,txt" -o -,txt)
    set_tests_properties(qt_image_preview PROPERTIES TIMEOUT 90)
    if(WIN32)
        target_link_libraries(xdzip-association-description-tests PRIVATE shell32)
    endif()
    add_test(NAME qt_association_descriptions COMMAND xdzip-association-description-tests
        -o "${CMAKE_CURRENT_BINARY_DIR}/association-descriptions-tests.log,txt" -o -,txt)
    add_executable(xdzip-browser-tests tests/browser_test.cpp
        tests/BrowserRequestTests.cpp tests/BrowserRequestTests.h
        tests/test_environment.cpp tests/test_environment.h)
    target_compile_definitions(xdzip-browser-tests PRIVATE XDZIP_TEST_TEMP_ROOT="${PROJECT_SOURCE_DIR}/build/test-tmp")
    if(APPLE)
        target_sources(xdzip-browser-tests PRIVATE tests/native_dialog_mac.mm)
        target_link_libraries(xdzip-browser-tests PRIVATE "-framework AppKit")
        set_target_properties(xdzip-browser-tests PROPERTIES OBJCXX_STANDARD 17 OBJCXX_STANDARD_REQUIRED ON)
        add_executable(xdzip-macos-association-probe tests/macos_association_probe.mm MacFileAssociations.mm)
        target_include_directories(xdzip-macos-association-probe PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
        target_link_libraries(xdzip-macos-association-probe PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets
            "-framework AppKit" "-framework UniformTypeIdentifiers")
        set_target_properties(xdzip-macos-association-probe PROPERTIES OBJCXX_STANDARD 17 OBJCXX_STANDARD_REQUIRED ON)
        add_executable(xdzip-macos-association-tests tests/macos_association_test.mm
            FileAssociations.cpp FileAssociations.h MacFileAssociations.mm)
        target_include_directories(xdzip-macos-association-tests PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
        target_link_libraries(xdzip-macos-association-tests PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets Qt6::Test
            "-framework AppKit" "-framework UniformTypeIdentifiers")
        set_target_properties(xdzip-macos-association-tests PROPERTIES OBJCXX_STANDARD 17 OBJCXX_STANDARD_REQUIRED ON)
        set_source_files_properties(tests/macos_association_test.mm PROPERTIES COMPILE_OPTIONS "-fobjc-arc;-fblocks")
        add_test(NAME qt_macos_association_settings COMMAND xdzip-macos-association-tests
            "$<TARGET_BUNDLE_DIR:xdzip-browser>/Contents/Info.plist")
        set_tests_properties(qt_macos_association_settings PROPERTIES TIMEOUT 30)
    endif()
    target_include_directories(xdzip-browser-tests PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
    set(XDZIP_M2_FIXTURE "${CMAKE_CURRENT_BINARY_DIR}/m2-flat.zip")
    add_custom_command(OUTPUT "${XDZIP_M2_FIXTURE}"
        COMMAND Python3::Interpreter "${PROJECT_SOURCE_DIR}/scripts/generate-m2-fixture.py" --output "${XDZIP_M2_FIXTURE}"
        DEPENDS "${PROJECT_SOURCE_DIR}/scripts/generate-m2-fixture.py" VERBATIM)
    set(XDZIP_M2_HIERARCHICAL "${CMAKE_CURRENT_BINARY_DIR}/m2-hierarchical.zip")
    add_custom_command(OUTPUT "${XDZIP_M2_HIERARCHICAL}"
        COMMAND Python3::Interpreter "${PROJECT_SOURCE_DIR}/scripts/generate-m2-fixture.py" --output "${XDZIP_M2_HIERARCHICAL}" --hierarchical
        DEPENDS "${PROJECT_SOURCE_DIR}/scripts/generate-m2-fixture.py" VERBATIM)
    add_custom_target(xdzip-m2-fixtures DEPENDS "${XDZIP_M2_FIXTURE}" "${XDZIP_M2_HIERARCHICAL}")
    set(XDZIP_READONLY_FIXTURES "${CMAKE_CURRENT_BINARY_DIR}/readonly-fixtures")
    file(GLOB XDZIP_CAB_ISO_INPUTS CONFIGURE_DEPENDS "${PROJECT_SOURCE_DIR}/tests/fixtures/cab-iso/*")
    set(XDZIP_READONLY_OUTPUTS)
    foreach(fixture named.cab iso9660.iso joliet.iso rock-ridge.iso udf.iso udf-regular.iso hybrid.iso)
        list(APPEND XDZIP_READONLY_OUTPUTS "${XDZIP_READONLY_FIXTURES}/${fixture}")
    endforeach()
    add_custom_command(OUTPUT ${XDZIP_READONLY_OUTPUTS}
        COMMAND Python3::Interpreter "${PROJECT_SOURCE_DIR}/scripts/prepare-cab-iso-fixtures.py" --output "${XDZIP_READONLY_FIXTURES}"
        DEPENDS "${PROJECT_SOURCE_DIR}/scripts/prepare-cab-iso-fixtures.py" ${XDZIP_CAB_ISO_INPUTS} VERBATIM)
    add_custom_target(xdzip-readonly-fixtures DEPENDS ${XDZIP_READONLY_OUTPUTS})
    add_dependencies(xdzip-browser-tests xdzip-readonly-fixtures)
    target_compile_definitions(xdzip-browser-tests PRIVATE XDZIP_READONLY_FIXTURES="${XDZIP_READONLY_FIXTURES}")
    add_dependencies(xdzip-browser-tests xdzip-m2-fixtures)
    target_compile_definitions(xdzip-browser-tests PRIVATE XDZIP_FIXTURES="${PROJECT_SOURCE_DIR}/tests/fixtures/zip" XDZIP_M2_FIXTURE="${XDZIP_M2_FIXTURE}" XDZIP_M2_HIERARCHICAL="${XDZIP_M2_HIERARCHICAL}")
    target_link_libraries(xdzip-browser-tests PRIVATE xdzip-desktop-sessionplugin xdzip-desktop-session
        Qt6::QuickControls2 Qt6::QuickTest Qt6::Test)
    add_test(NAME qt_zip_browser COMMAND Python3::Interpreter
        "${PROJECT_SOURCE_DIR}/scripts/run-browser-suite.py"
        --executable "$<TARGET_FILE:xdzip-browser-tests>"
        --output "${CMAKE_CURRENT_BINARY_DIR}/browser-tests.log")
    # Whole-suite ceiling includes large scans and many event-loop waits. Individual
    # QTRY assertions keep their own deadlines; a slow host must report the assertion.
    # All discovered functions run within this single ceiling. Offscreen workers
    # have isolated settings; native and explicitly shared settings run serially.
    if(APPLE)
        # Hosted Apple Silicon runs have completed within a few seconds of 240.
        set_tests_properties(qt_zip_browser PROPERTIES TIMEOUT 300)
    else()
        set_tests_properties(qt_zip_browser PROPERTIES TIMEOUT 240)
    endif()
    add_test(NAME qt_browser_load COMMAND xdzip-browser --smoke-test)
    add_executable(xdzip-update-tests tests/update_controller_test.cpp ${xdzip_update_sources})
    target_include_directories(xdzip-update-tests PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
    target_compile_definitions(xdzip-update-tests PRIVATE XDZIP_UPDATE_FIXTURES="${PROJECT_SOURCE_DIR}/tests/fixtures/update")
    target_link_libraries(xdzip-update-tests PRIVATE xdzip-desktop-sessionplugin xdzip-desktop-session xdzip-ui-resourcesplugin
        Qt6::QuickControls2 Qt6::Test)
    if(APPLE)
        target_sources(xdzip-update-tests PRIVATE MacUpdateProvider.mm)
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
            file(READ "${XDZIP_MACOS_BUNDLE_PLIST}" update_probe_info)
            string(REPLACE "<key>XdzipFinderSyncProtocol</key><integer>1</integer>" ""
                update_probe_info "${update_probe_info}")
            file(WRITE "${CMAKE_CURRENT_BINARY_DIR}/UpdateProbe-Info.plist.in" "${update_probe_info}")
            add_executable(xdzip-mac-update-probe MACOSX_BUNDLE tests/mac_update_probe.mm
                ${xdzip_update_sources} MacUpdateProvider.mm)
            target_include_directories(xdzip-mac-update-probe PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
            target_link_libraries(xdzip-mac-update-probe PRIVATE xdzip-desktop-sessionplugin xdzip-desktop-session xdzip-ui-resourcesplugin
                Qt6::QuickControls2 Qt6::Test
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
                MACOSX_BUNDLE_INFO_PLIST "${CMAKE_CURRENT_BINARY_DIR}/UpdateProbe-Info.plist.in")
        endif()
    endif()
    add_test(NAME qt_updates COMMAND xdzip-update-tests -v1 -o "${CMAKE_CURRENT_BINARY_DIR}/update-tests.log,txt")
    set_tests_properties(qt_updates PROPERTIES TIMEOUT 60)
endif()
foreach(target xdzip-desktop-session xdzip-browser xdzip-browser-tests xdzip-update-tests xdzip-association-description-tests xdzip-macos-association-probe xdzip-macos-association-tests xdzip-image-tests xdzip-image-fault)
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
        target_link_libraries(${target} PRIVATE xdzip-ui-resourcesplugin xdzip-password-accessibility)
    endif()
endforeach()

include("${PROJECT_SOURCE_DIR}/cmake/NativeCodecDeployment.cmake")

if(WIN32)
    add_library(xdzip-shell-extension SHARED shell/WindowsContextMenu.cpp shell/WindowsContextMenu.def)
    configure_file(shell/WindowsMenuIcons.rc.in "${CMAKE_CURRENT_BINARY_DIR}/xdzip-shell-icons.rc" @ONLY)
    target_sources(xdzip-shell-extension PRIVATE "${CMAKE_CURRENT_BINARY_DIR}/xdzip-shell-icons.rc")
    set_target_properties(xdzip-shell-extension PROPERTIES OUTPUT_NAME xdzip-shell)
    target_link_libraries(xdzip-shell-extension PRIVATE shell32 ole32 uuid advapi32 bcrypt version gdi32 user32 windowsapp)
    if(MSVC)
        target_compile_options(xdzip-shell-extension PRIVATE /W4 /WX /utf-8)
        set_property(TARGET xdzip-shell-extension PROPERTY MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
    endif()
    add_dependencies(xdzip-browser xdzip-shell-extension)
elseif(APPLE)
    configure_file(shell/Finder-Info.plist.in "${CMAKE_CURRENT_BINARY_DIR}/Finder-Info.plist" @ONLY)
    configure_file(shell/Relay-Info.plist.in "${CMAKE_CURRENT_BINARY_DIR}/Relay-Info.plist" @ONLY)
    add_executable(xdzip-finder-sync shell/MacFinderSync.mm)
    add_executable(xdzip-shell-relay shell/MacFinderRelay.mm)
    foreach(finder_target xdzip-finder-sync xdzip-shell-relay)
        set_target_properties(${finder_target} PROPERTIES OBJCXX_STANDARD 17 OBJCXX_STANDARD_REQUIRED ON)
        target_compile_options(${finder_target} PRIVATE -fobjc-arc -fblocks -Wall -Wextra -Werror)
        target_link_libraries(${finder_target} PRIVATE "-framework AppKit" "-framework UniformTypeIdentifiers")
    endforeach()
    target_compile_options(xdzip-finder-sync PRIVATE -fapplication-extension)
    target_link_options(xdzip-finder-sync PRIVATE -e _NSExtensionMain)
    target_link_libraries(xdzip-finder-sync PRIVATE "-framework FinderSync")
    set(finder_bundle "${XDZIP_APP_BUNDLE}/PlugIns/xdzip-finder-sync.appex/Contents")
    set(relay_bundle "${XDZIP_APP_BUNDLE}/Helpers/xdzip-shell-relay.app/Contents")
    xdzip_stage_file("${finder_bundle}/MacOS/xdzip-finder-sync" "$<TARGET_FILE:xdzip-finder-sync>")
    xdzip_stage_file("${relay_bundle}/MacOS/xdzip-shell-relay" "$<TARGET_FILE:xdzip-shell-relay>")
    xdzip_stage_file("${finder_bundle}/Info.plist" "${CMAKE_CURRENT_BINARY_DIR}/Finder-Info.plist")
    xdzip_stage_file("${relay_bundle}/Info.plist" "${CMAKE_CURRENT_BINARY_DIR}/Relay-Info.plist")
    xdzip_stage_file("${finder_bundle}/Resources/Finder.entitlements" "${CMAKE_CURRENT_SOURCE_DIR}/shell/Finder.entitlements")
    add_executable(xdzip-shell-service shell/MacService.mm)
    set_target_properties(xdzip-shell-service PROPERTIES OBJCXX_STANDARD 17 OBJCXX_STANDARD_REQUIRED ON)
    set_source_files_properties(shell/MacService.mm MacShellService.mm PROPERTIES COMPILE_OPTIONS "-fobjc-arc;-fblocks")
    target_link_libraries(xdzip-shell-service PRIVATE "-framework AppKit" "-framework UniformTypeIdentifiers")
    target_compile_options(xdzip-shell-service PRIVATE -Wall -Wextra -Werror)
    target_sources(xdzip-browser PRIVATE MacShellService.mm MacFinderIntegration.mm)
    set(service_bundle "${XDZIP_APP_BUNDLE}/Helpers/xdzip.service/Contents")
    xdzip_stage_file("${service_bundle}/MacOS/xdzip-shell-service" "$<TARGET_FILE:xdzip-shell-service>")
    xdzip_stage_file("${service_bundle}/Info.plist" "${CMAKE_CURRENT_SOURCE_DIR}/shell/Service-Info.plist")
    set(mac_outputs
        "${finder_bundle}/MacOS/xdzip-finder-sync" "${finder_bundle}/Info.plist"
        "${finder_bundle}/Resources/Finder.entitlements"
        "${relay_bundle}/MacOS/xdzip-shell-relay" "${relay_bundle}/Info.plist"
        "${service_bundle}/MacOS/xdzip-shell-service" "${service_bundle}/Info.plist")
    foreach(kind service bundle)
        set(resources "${XDZIP_APP_BUNDLE}/Resources")
        if(kind STREQUAL "service")
            set(resources "${service_bundle}/Resources")
        endif()
        file(GLOB_RECURSE localized RELATIVE "${XDZIP_PACKAGING_TRANSLATIONS}/${kind}"
            "${XDZIP_PACKAGING_TRANSLATIONS}/${kind}/*.strings")
        foreach(path IN LISTS localized)
            set(destination "${resources}/${path}")
            xdzip_stage_file("${destination}" "${XDZIP_PACKAGING_TRANSLATIONS}/${kind}/${path}")
            list(APPEND mac_outputs "${destination}")
        endforeach()
    endforeach()
    add_custom_target(xdzip-macos-shell-stage DEPENDS ${mac_outputs})
    add_dependencies(xdzip-browser xdzip-macos-shell-stage)
elseif(UNIX)
    find_package(PkgConfig REQUIRED)
    pkg_check_modules(NAUTILUS4 REQUIRED IMPORTED_TARGET libnautilus-extension-4)
    add_library(xdzip-nautilus MODULE shell/NautilusMenu.cpp)
    target_link_libraries(xdzip-nautilus PRIVATE PkgConfig::NAUTILUS4)
    target_compile_options(xdzip-nautilus PRIVATE -Wall -Wextra -Werror)
    add_dependencies(xdzip-browser xdzip-nautilus)
endif()

if(BUILD_TESTING)
    add_executable(xdzip-shell-tests tests/shell_actions_test.cpp ShellIntegration.cpp ShellIntegration.h
        tests/test_environment.cpp tests/test_environment.h)
    target_include_directories(xdzip-shell-tests PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
    target_compile_definitions(xdzip-shell-tests PRIVATE XDZIP_TEST_TEMP_ROOT="${PROJECT_SOURCE_DIR}/build/test-tmp" XDZIP_SHELL_FIXTURES="${PROJECT_SOURCE_DIR}/tests/fixtures/extract")
    target_link_libraries(xdzip-shell-tests PRIVATE xdzip-desktop-sessionplugin xdzip-desktop-session xdzip-ui-resourcesplugin
        xdzip-password-accessibility Qt6::QuickControls2 Qt6::Test)
    if(APPLE)
        target_sources(xdzip-shell-tests PRIVATE MacShellService.mm MacFinderIntegration.mm)
        set_target_properties(xdzip-shell-tests PROPERTIES OBJCXX_STANDARD 17 OBJCXX_STANDARD_REQUIRED ON)
        target_link_libraries(xdzip-shell-tests PRIVATE "-framework AppKit")
    elseif(WIN32)
        target_link_libraries(xdzip-shell-tests PRIVATE bcrypt advapi32 ole32 shell32 shlwapi version windowsapp)
    elseif(CMAKE_SYSTEM_NAME STREQUAL "Linux")
        target_link_libraries(xdzip-shell-tests PRIVATE X11::X11)
    endif()
    add_dependencies(xdzip-shell-tests xdzip-browser)
    target_compile_definitions(xdzip-shell-tests PRIVATE XDZIP_SHELL_PRODUCT="$<TARGET_FILE:xdzip-browser>")
    add_test(NAME qt_shell_actions COMMAND xdzip-shell-tests -o "${CMAKE_CURRENT_BINARY_DIR}/shell-actions-tests.log,txt" -o -,txt)
    set_tests_properties(qt_shell_actions PROPERTIES TIMEOUT 90 ENVIRONMENT "QT_QPA_PLATFORM=offscreen;QT_QUICK_BACKEND=software")
endif()

if(BUILD_TESTING AND WIN32)
    add_executable(xdzip-windows-frame-policy-test tests/windows_frame_policy_test.cpp)
    add_test(NAME windows_frame_policy COMMAND xdzip-windows-frame-policy-test)
    add_executable(xdzip-windows-frame-test tests/windows_frame_test.cpp WindowsFrame.cpp WindowsFrame.h)
    target_link_libraries(xdzip-windows-frame-test PRIVATE Qt6::Core Qt6::Gui Qt6::Qml dwmapi)
    add_test(NAME native_windows_frame_calculation COMMAND xdzip-windows-frame-test)
    set_tests_properties(native_windows_frame_calculation PROPERTIES ENVIRONMENT "QT_QPA_PLATFORM=windows;QT_QUICK_BACKEND=software")
    add_executable(xdzip-windows-shell-test shell/WindowsMenuTest.cpp)
    target_link_libraries(xdzip-windows-shell-test PRIVATE shell32 ole32 uuid advapi32 bcrypt version gdi32 windowsapp)
    if(MSVC)
        target_compile_options(xdzip-windows-shell-test PRIVATE /utf-8)
    endif()
    add_dependencies(xdzip-windows-shell-test xdzip-shell-extension)
    add_test(NAME native_windows_shell_menu COMMAND xdzip-windows-shell-test "$<TARGET_FILE:xdzip-shell-extension>")
elseif(BUILD_TESTING AND UNIX AND NOT APPLE)
    pkg_check_modules(GMODULE REQUIRED IMPORTED_TARGET gmodule-2.0)
    add_executable(xdzip-nautilus-menu-test shell/NautilusMenuTest.cpp)
    target_link_libraries(xdzip-nautilus-menu-test PRIVATE PkgConfig::NAUTILUS4 PkgConfig::GMODULE)
    target_compile_options(xdzip-nautilus-menu-test PRIVATE -Wall -Wextra -Werror)
    add_dependencies(xdzip-nautilus-menu-test xdzip-nautilus)
    add_test(NAME native_nautilus_shell_menu COMMAND xdzip-nautilus-menu-test "$<TARGET_FILE:xdzip-nautilus>")
elseif(BUILD_TESTING AND APPLE)
    add_executable(xdzip-mac-shell-capability-test shell/MacCapabilityTest.mm)
    set_target_properties(xdzip-mac-shell-capability-test PROPERTIES OBJCXX_STANDARD 17 OBJCXX_STANDARD_REQUIRED ON)
    target_link_libraries(xdzip-mac-shell-capability-test PRIVATE "-framework Foundation")
    target_compile_options(xdzip-mac-shell-capability-test PRIVATE -Wall -Wextra -Werror)
    add_dependencies(xdzip-mac-shell-capability-test xdzip-browser)
    add_test(NAME native_mac_shell_capability COMMAND xdzip-mac-shell-capability-test "${PROJECT_BINARY_DIR}/shell-capability" "$<TARGET_BUNDLE_DIR:xdzip-browser>")
    add_executable(xdzip-mac-finder-test shell/MacFinderTest.mm MacFinderIntegration.mm)
    set_target_properties(xdzip-mac-finder-test PROPERTIES OBJCXX_STANDARD 17 OBJCXX_STANDARD_REQUIRED ON)
    target_compile_options(xdzip-mac-finder-test PRIVATE -fobjc-arc -fblocks -Wall -Wextra -Werror)
    target_link_libraries(xdzip-mac-finder-test PRIVATE Qt6::Core "-framework AppKit" "-framework UniformTypeIdentifiers")
    add_dependencies(xdzip-mac-finder-test xdzip-browser)
    add_test(NAME native_mac_finder COMMAND xdzip-mac-finder-test "${PROJECT_SOURCE_DIR}/build/test-tmp" "$<TARGET_BUNDLE_DIR:xdzip-browser>")
endif()
