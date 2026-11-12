set(XDZIP_MACOS_HANDLER_RANK "None" CACHE STRING "macOS document handling rank: None for development/test copies, Alternate for the primary application")
set_property(CACHE XDZIP_MACOS_HANDLER_RANK PROPERTY STRINGS None Alternate)
if(NOT XDZIP_MACOS_HANDLER_RANK MATCHES "^(None|Alternate)$")
    message(FATAL_ERROR "XDZIP_MACOS_HANDLER_RANK must be None or Alternate")
endif()

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
add_dependencies(xdzip-ui-resources xdzip-image-preview)
add_dependencies(xdzip-ui-resources xdzip-build-info)
target_include_directories(xdzip-ui-resources PUBLIC "${XDZIP_BUILD_INFO_DIR}")
qt_add_qml_module(xdzip-ui-resources
    URI Xdzip.Settings VERSION 1.0
    DEPENDENCIES QtQuick
    OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/qml/Xdzip/Settings"
    SOURCES PreviewImages.h PreviewImages.cpp ImagePreviewProcess.h ImagePreviewProcess.cpp UiPreferences.h UiPreferences.cpp UiFonts.h WindowsFrame.h WindowsFrame.cpp BuildInfo.h BuildInfo.cpp)
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
qt_add_executable(xdzip-browser browser_main.cpp ShellIntegration.h ShellIntegration.cpp)
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
set(XDZIP_CLI_PROFILE debug)
if(CMAKE_BUILD_TYPE STREQUAL "Release")
    set(XDZIP_CLI_PROFILE release)
endif()
set(XDZIP_CLI_BUILD "${CMAKE_BINARY_DIR}/cli")
set(XDZIP_CLI_BUNDLE "${CMAKE_CURRENT_BINARY_DIR}/cli")
if(APPLE)
    set(XDZIP_CLI_BUNDLE "${CMAKE_CURRENT_BINARY_DIR}/xdzip.app/Contents/Helpers/cli")
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
    enable_language(OBJCXX)
    include(${PROJECT_SOURCE_DIR}/cmake/Sparkle.cmake)
    set_target_properties(xdzip-browser PROPERTIES OBJCXX_STANDARD 17
        OBJCXX_STANDARD_REQUIRED ON OBJCXX_EXTENSIONS OFF)
    target_sources(xdzip-browser PRIVATE MacTitlebar.mm MacWindowActivation.mm MacFileAssociations.mm MacQuitMenu.mm)
    set_source_files_properties(MacQuitMenu.mm PROPERTIES COMPILE_OPTIONS "-fobjc-arc")
    set_source_files_properties(MacFileAssociations.mm PROPERTIES COMPILE_OPTIONS "-fobjc-arc;-fblocks")
    target_link_libraries(xdzip-browser PRIVATE "-framework UniformTypeIdentifiers")
    target_sources(xdzip-browser PRIVATE MacUpdateProvider.mm MacUpdateProvider.h)
    set_source_files_properties(MacUpdateProvider.mm PROPERTIES COMPILE_OPTIONS "-fobjc-arc;-fblocks;-Werror=protocol")
    target_link_libraries(xdzip-browser PRIVATE "${XDZIP_SPARKLE_FRAMEWORK}")
    set_property(TARGET xdzip-browser APPEND PROPERTY BUILD_RPATH "${XDZIP_SPARKLE_SDK}")
    target_link_libraries(xdzip-browser PRIVATE "-framework AppKit")
    set_target_properties(xdzip-browser PROPERTIES MACOSX_BUNDLE TRUE
        MACOSX_BUNDLE_GUI_IDENTIFIER "org.xdzip.desktop"
        MACOSX_BUNDLE_INFO_PLIST "${CMAKE_CURRENT_SOURCE_DIR}/packaging/macos/Info.plist.in"
        MACOSX_BUNDLE_BUNDLE_NAME "xdzip"
        MACOSX_BUNDLE_COPYRIGHT "Copyright (C) 2026 xiaodaozhi"
        MACOSX_BUNDLE_ICON_FILE "xdzip.icns"
        MACOSX_BUNDLE_SHORT_VERSION_STRING "${PROJECT_VERSION}"
        MACOSX_BUNDLE_BUNDLE_VERSION "${XDZIP_BUILD_NUMBER}"
        MACOSX_BUNDLE_LONG_VERSION_STRING "${XDZIP_DISPLAY_VERSION}")
    set_source_files_properties("${XDZIP_BRAND_ASSETS}/xdzip.icns" PROPERTIES
        MACOSX_PACKAGE_LOCATION "Resources")
    target_sources(xdzip-browser PRIVATE "${XDZIP_BRAND_ASSETS}/xdzip.icns")
    foreach(icon zip tar tar-gz gz 7z rar cab iso udf)
        set(document_icon "${PROJECT_SOURCE_DIR}/design/assets/file-associations/xdzip-${icon}.icns")
        set_source_files_properties("${document_icon}" PROPERTIES MACOSX_PACKAGE_LOCATION "Resources")
        target_sources(xdzip-browser PRIVATE "${document_icon}")
    endforeach()
endif()
qt_add_qml_module(xdzip-browser
    URI Xdzip.Native VERSION 1.0
    DEPENDENCIES QtQml.Models
    OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/qml/Xdzip/Native"
    SOURCES ShellBatchController.h ShellBatchController.cpp ShellActionRequest.h ShellActionRequest.cpp CliRequestBroker.h CliRequestBroker.cpp BrowserController.h BrowserController.cpp RuntimeConnection.h RuntimeConnection.cpp DesktopIntegration.h DesktopIntegration.cpp ApplicationSessions.h ApplicationSessions.cpp SingleInstance.h SingleInstance.cpp UpdateController.h UpdateController.cpp UpdateHandoff.h UpdateHandoff.cpp FileAssociations.h FileAssociations.cpp)
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
    install(FILES packaging/linux/xdzip-mime.xml DESTINATION "${CMAKE_INSTALL_DATADIR}/mime/packages")
    install(FILES "${XDZIP_BRAND_ASSETS}/xdzip.png"
        DESTINATION "${CMAKE_INSTALL_DATADIR}/icons/hicolor/512x512/apps")
endif()
if(BUILD_TESTING)
    add_executable(xdzip-association-description-tests tests/association_descriptions_test.cpp
        WindowsAssociationDescriptions.cpp WindowsAssociationDescriptions.h)
    target_include_directories(xdzip-association-description-tests PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
    find_package(Qt6 6.8 REQUIRED COMPONENTS Test)
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
    add_executable(xdzip-browser-tests tests/browser_test.cpp tests/test_environment.cpp tests/test_environment.h BrowserController.cpp BrowserController.h RuntimeConnection.cpp RuntimeConnection.h DesktopIntegration.h DesktopIntegration.cpp ApplicationSessions.h ApplicationSessions.cpp SingleInstance.h SingleInstance.cpp)
    target_compile_definitions(xdzip-browser-tests PRIVATE XDZIP_TEST_TEMP_ROOT="${PROJECT_SOURCE_DIR}/build/test-tmp")
    if(APPLE)
        target_sources(xdzip-browser-tests PRIVATE MacTitlebar.mm MacWindowActivation.mm MacQuitMenu.mm tests/native_dialog_mac.mm)
        target_link_libraries(xdzip-browser-tests PRIVATE "-framework AppKit")
        set_target_properties(xdzip-browser-tests PROPERTIES OBJCXX_STANDARD 17 OBJCXX_STANDARD_REQUIRED ON)
        add_executable(xdzip-macos-association-probe tests/macos_association_probe.mm MacFileAssociations.mm)
        target_include_directories(xdzip-macos-association-probe PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
        target_link_libraries(xdzip-macos-association-probe PRIVATE Qt6::Core Qt6::Gui Qt6::Widgets
            "-framework AppKit" "-framework UniformTypeIdentifiers")
        set_target_properties(xdzip-macos-association-probe PROPERTIES OBJCXX_STANDARD 17 OBJCXX_STANDARD_REQUIRED ON)
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
    target_link_libraries(xdzip-browser-tests PRIVATE xdzip-reader-rust
        Qt6::Widgets Qt6::Network Qt6::Quick Qt6::QuickControls2 Qt6::QuickTest Qt6::Test)
    add_test(NAME qt_zip_browser COMMAND Python3::Interpreter
        "${PROJECT_SOURCE_DIR}/scripts/run-browser-suite.py"
        --executable "$<TARGET_FILE:xdzip-browser-tests>"
        --output "${CMAKE_CURRENT_BINARY_DIR}/browser-tests.log")
    # Whole-suite ceiling includes large scans and many event-loop waits. Individual
    # QTRY assertions keep their own deadlines; a slow host must report the assertion.
    # All discovered functions run within this single ceiling. Offscreen workers
    # have isolated settings; native and explicitly shared settings run serially.
    set_tests_properties(qt_zip_browser PROPERTIES TIMEOUT 240)
    add_test(NAME qt_browser_load COMMAND xdzip-browser --smoke-test)
    add_executable(xdzip-update-tests tests/update_controller_test.cpp
        CliRequestBroker.cpp CliRequestBroker.h
        UpdateController.cpp UpdateController.h UpdateHandoff.cpp UpdateHandoff.h
        BrowserController.cpp BrowserController.h RuntimeConnection.cpp RuntimeConnection.h
        DesktopIntegration.cpp DesktopIntegration.h ApplicationSessions.cpp ApplicationSessions.h
        SingleInstance.cpp SingleInstance.h)
    target_include_directories(xdzip-update-tests PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
    target_compile_definitions(xdzip-update-tests PRIVATE XDZIP_UPDATE_FIXTURES="${PROJECT_SOURCE_DIR}/tests/fixtures/update")
    target_link_libraries(xdzip-update-tests PRIVATE xdzip-reader-rust xdzip-ui-resourcesplugin
        Qt6::Widgets Qt6::Network Qt6::Quick Qt6::QuickControls2 Qt6::Test)
    if(APPLE)
        target_sources(xdzip-update-tests PRIVATE MacTitlebar.mm MacWindowActivation.mm MacQuitMenu.mm MacUpdateProvider.mm)
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
foreach(target xdzip-browser xdzip-browser-tests xdzip-update-tests xdzip-association-description-tests xdzip-macos-association-probe xdzip-image-tests xdzip-image-fault)
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

# External file transfers share the controller and immutable private-copy lifetime.
foreach(transfer_target xdzip-browser xdzip-browser-tests xdzip-update-tests xdzip-mac-update-probe)
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

include("${PROJECT_SOURCE_DIR}/cmake/NativeCodecDeployment.cmake")

foreach(shell_test xdzip-browser-tests xdzip-update-tests)
    if(TARGET ${shell_test})
        target_sources(${shell_test} PRIVATE ShellBatchController.h ShellBatchController.cpp ShellActionRequest.h ShellActionRequest.cpp)
    endif()
endforeach()

if(WIN32)
    add_library(xdzip-shell-extension SHARED shell/WindowsContextMenu.cpp shell/WindowsContextMenu.def)
    set_target_properties(xdzip-shell-extension PROPERTIES OUTPUT_NAME xdzip-shell)
    target_link_libraries(xdzip-shell-extension PRIVATE shell32 ole32 uuid advapi32 bcrypt version)
    if(MSVC)
        target_compile_options(xdzip-shell-extension PRIVATE /W4 /WX /utf-8)
        set_property(TARGET xdzip-shell-extension PROPERTY MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
    endif()
    add_dependencies(xdzip-browser xdzip-shell-extension)
elseif(APPLE)
    add_executable(xdzip-shell-service shell/MacService.mm)
    set_target_properties(xdzip-shell-service PROPERTIES OBJCXX_STANDARD 17 OBJCXX_STANDARD_REQUIRED ON)
    set_source_files_properties(shell/MacService.mm MacShellService.mm PROPERTIES COMPILE_OPTIONS "-fobjc-arc;-fblocks")
    target_link_libraries(xdzip-shell-service PRIVATE "-framework AppKit" "-framework UniformTypeIdentifiers")
    target_compile_options(xdzip-shell-service PRIVATE -Wall -Wextra -Werror)
    target_sources(xdzip-browser PRIVATE MacShellService.mm)
    add_dependencies(xdzip-browser xdzip-shell-service)
    set(service_bundle "$<TARGET_BUNDLE_DIR:xdzip-browser>/Contents/Helpers/xdzip.service/Contents")
    add_custom_command(TARGET xdzip-browser POST_BUILD
        COMMAND "${CMAKE_COMMAND}" -E make_directory "${service_bundle}/MacOS" "${service_bundle}/Resources/en.lproj" "${service_bundle}/Resources/zh-Hans.lproj"
        COMMAND "${CMAKE_COMMAND}" -E copy_if_different "$<TARGET_FILE:xdzip-shell-service>" "${service_bundle}/MacOS/xdzip-shell-service"
        COMMAND "${CMAKE_COMMAND}" -E copy_if_different "${CMAKE_CURRENT_SOURCE_DIR}/shell/Service-Info.plist" "${service_bundle}/Info.plist"
        COMMAND "${CMAKE_COMMAND}" -E copy_if_different "${CMAKE_CURRENT_SOURCE_DIR}/shell/ServicesMenu-en.strings" "${service_bundle}/Resources/en.lproj/ServicesMenu.strings"
        COMMAND "${CMAKE_COMMAND}" -E copy_if_different "${CMAKE_CURRENT_SOURCE_DIR}/shell/ServicesMenu-zh.strings" "${service_bundle}/Resources/zh-Hans.lproj/ServicesMenu.strings" VERBATIM)
elseif(UNIX)
    find_package(PkgConfig REQUIRED)
    pkg_check_modules(NAUTILUS4 REQUIRED IMPORTED_TARGET libnautilus-extension-4)
    add_library(xdzip-nautilus MODULE shell/NautilusMenu.cpp)
    target_link_libraries(xdzip-nautilus PRIVATE PkgConfig::NAUTILUS4)
    target_compile_options(xdzip-nautilus PRIVATE -Wall -Wextra -Werror)
    add_dependencies(xdzip-browser xdzip-nautilus)
endif()

if(BUILD_TESTING)
    get_target_property(shell_test_sources xdzip-browser-tests SOURCES)
    list(FILTER shell_test_sources EXCLUDE REGEX "tests/browser_test\\.cpp$")
    add_executable(xdzip-shell-tests tests/shell_actions_test.cpp ShellIntegration.cpp ShellIntegration.h ${shell_test_sources})
    target_include_directories(xdzip-shell-tests PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
    target_compile_definitions(xdzip-shell-tests PRIVATE XDZIP_TEST_TEMP_ROOT="${PROJECT_SOURCE_DIR}/build/test-tmp" XDZIP_SHELL_FIXTURES="${PROJECT_SOURCE_DIR}/tests/fixtures/extract")
    target_link_libraries(xdzip-shell-tests PRIVATE xdzip-reader-rust xdzip-ui-resourcesplugin xdzip-password-accessibility Qt6::Widgets Qt6::Network Qt6::Quick Qt6::QuickControls2 Qt6::Test)
    if(APPLE)
        target_sources(xdzip-shell-tests PRIVATE MacShellService.mm)
        set_target_properties(xdzip-shell-tests PROPERTIES OBJCXX_STANDARD 17 OBJCXX_STANDARD_REQUIRED ON)
        target_link_libraries(xdzip-shell-tests PRIVATE "-framework AppKit")
    elseif(WIN32)
        target_link_libraries(xdzip-shell-tests PRIVATE bcrypt advapi32 ole32 shell32 shlwapi version)
    endif()
    add_dependencies(xdzip-shell-tests xdzip-browser)
    target_compile_definitions(xdzip-shell-tests PRIVATE XDZIP_SHELL_PRODUCT="$<TARGET_FILE:xdzip-browser>")
    add_test(NAME qt_shell_actions COMMAND xdzip-shell-tests -o "${CMAKE_CURRENT_BINARY_DIR}/shell-actions-tests.log,txt" -o -,txt)
    set_tests_properties(qt_shell_actions PROPERTIES TIMEOUT 90 ENVIRONMENT "QT_QPA_PLATFORM=offscreen;QT_QUICK_BACKEND=software")
endif()

if(BUILD_TESTING AND WIN32)
    add_executable(xdzip-windows-shell-test shell/WindowsMenuTest.cpp)
    target_link_libraries(xdzip-windows-shell-test PRIVATE shell32 ole32 uuid advapi32 bcrypt)
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
endif()
