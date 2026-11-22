# SPDX-License-Identifier: Apache-2.0
# Retain the upstream version strings for binary verification under MSVC.
if(MSVC)
    if(CMAKE_PROJECT_NAME STREQUAL "QtBase")
        cmake_language(DEFER CALL target_link_options Gui PRIVATE "/INCLUDE:png_get_copyright")
    elseif(CMAKE_PROJECT_NAME STREQUAL "QtImageFormats")
        cmake_language(DEFER CALL target_link_options QTiffPlugin PRIVATE "/INCLUDE:TIFFGetVersion")
    endif()
endif()
