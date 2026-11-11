# SPDX-License-Identifier: Apache-2.0
# Add only storage for SDK-exported constexpr constants, never runtime behavior.
if(CMAKE_PROJECT_NAME STREQUAL "QtBase")
    function(xdzip_retain_network_abi)
        if(NOT TARGET Network OR NOT EXISTS "${XDZIP_QT_NETWORK_ABI_SOURCE}")
            message(FATAL_ERROR "Missing Network target or pinned ABI retention input")
        endif()
        set(abi_source "${CMAKE_BINARY_DIR}/xdzip-network-abi-retain.cpp")
        configure_file("${XDZIP_QT_NETWORK_ABI_SOURCE}" "${abi_source}" COPYONLY)
        set_source_files_properties("${abi_source}" PROPERTIES GENERATED TRUE)
        target_sources(Network PRIVATE "${abi_source}")
    endfunction()
    cmake_language(DEFER CALL xdzip_retain_network_abi)
endif()
