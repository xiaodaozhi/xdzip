find_package(Qt6 6.8 REQUIRED COMPONENTS Core Gui Svg)
add_executable(xdzip-image-preview main.cpp)
target_link_libraries(xdzip-image-preview PRIVATE Qt6::Core Qt6::Gui Qt6::Svg)
target_include_directories(xdzip-image-preview PRIVATE "${PROJECT_SOURCE_DIR}/apps/desktop")
if(MSVC)
    target_compile_options(xdzip-image-preview PRIVATE /W4 /WX /utf-8)
else()
    target_compile_options(xdzip-image-preview PRIVATE -Wall -Wextra -Werror)
endif()
set_target_properties(xdzip-image-preview PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/apps/desktop")
if(APPLE)
    set(preview_bundle_directory "${CMAKE_BINARY_DIR}/apps/desktop/xdzip.app/Contents/MacOS")
    add_custom_command(TARGET xdzip-image-preview POST_BUILD
        COMMAND "${CMAKE_COMMAND}" -E make_directory "${preview_bundle_directory}"
        COMMAND "${CMAKE_COMMAND}" -E copy_if_different "$<TARGET_FILE:xdzip-image-preview>"
            "${preview_bundle_directory}/xdzip-image-preview")
endif()
