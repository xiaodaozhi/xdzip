# SPDX-License-Identifier: Apache-2.0
# Keep storage for the SDK's exported QtCore metatype constants in QtGui.
if(CMAKE_PROJECT_NAME STREQUAL "QtBase")
    function(xdzip_retain_gui_abi)
        if(NOT TARGET Gui OR NOT EXISTS "${XDZIP_QT_GUI_ABI_SOURCE}")
            message(FATAL_ERROR "Missing QtGui target or ABI retention source")
        endif()
        set(abi_source "${CMAKE_BINARY_DIR}/xdzip-gui-abi-retain.cpp")
        configure_file("${XDZIP_QT_GUI_ABI_SOURCE}" "${abi_source}" COPYONLY)
        set_source_files_properties("${abi_source}" PROPERTIES GENERATED TRUE)
        target_sources(Gui PRIVATE "${abi_source}")
    endfunction()
    cmake_language(DEFER CALL xdzip_retain_gui_abi)
endif()
