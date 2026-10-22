# cmake/UpdateHelper.cmake

# Keep the replacement runner independent of Qt and, on Windows, the installed CRT.
if(WIN32 OR (UNIX AND NOT APPLE))
    find_program(XDZIP_UPDATE_CARGO cargo REQUIRED)
    find_program(XDZIP_UPDATE_RUSTC rustc REQUIRED)
    execute_process(COMMAND "${XDZIP_UPDATE_RUSTC}" -vV OUTPUT_VARIABLE update_rust_version
        COMMAND_ERROR_IS_FATAL ANY)
    string(REGEX MATCH "host: ([^\n\r]+)" update_host_match "${update_rust_version}")
    set(update_host "${CMAKE_MATCH_1}")
    set(update_profile debug)
    set(update_cargo_profile dev)
    if(CMAKE_BUILD_TYPE STREQUAL "Release")
        set(update_profile release)
        set(update_cargo_profile release)
    endif()
    set(update_target_dir "${CMAKE_CURRENT_BINARY_DIR}/update-helper-rust")
    set(update_binary "${update_target_dir}/${update_host}/${update_profile}/xdzip-update-helper${CMAKE_EXECUTABLE_SUFFIX}")
    set(update_environment)
    if(WIN32)
        list(APPEND update_environment "--unset=CARGO_ENCODED_RUSTFLAGS" "RUSTFLAGS=-C target-feature=+crt-static")
    endif()
    file(GLOB_RECURSE update_sources CONFIGURE_DEPENDS
        "${PROJECT_SOURCE_DIR}/crates/xdzip-update-helper/*.rs"
        "${PROJECT_SOURCE_DIR}/crates/xdzip-update/*.rs"
        "${PROJECT_SOURCE_DIR}/crates/xdzip-io/*.rs")
    add_custom_command(OUTPUT "${update_binary}"
        COMMAND "${CMAKE_COMMAND}" -E env ${update_environment} "${XDZIP_UPDATE_CARGO}" build
            --locked -p xdzip-update-helper --manifest-path "${PROJECT_SOURCE_DIR}/Cargo.toml"
            --target-dir "${update_target_dir}" --target "${update_host}" --profile "${update_cargo_profile}"
        DEPENDS ${update_sources} "${PROJECT_SOURCE_DIR}/resources/update-trust.json"
            "${PROJECT_SOURCE_DIR}/crates/xdzip-update-helper/Cargo.toml"
            "${PROJECT_SOURCE_DIR}/Cargo.lock" "${PROJECT_SOURCE_DIR}/Cargo.toml"
        WORKING_DIRECTORY "${PROJECT_SOURCE_DIR}" VERBATIM)
    add_custom_target(xdzip-update-helper-build DEPENDS "${update_binary}")
    add_dependencies(xdzip-browser xdzip-update-helper-build)
    set_property(TARGET xdzip-browser APPEND PROPERTY LINK_DEPENDS "${update_binary}")
    add_custom_command(TARGET xdzip-browser POST_BUILD
        COMMAND "${CMAKE_COMMAND}" -E copy_if_different "${update_binary}"
            "$<TARGET_FILE_DIR:xdzip-browser>/xdzip-update-helper${CMAKE_EXECUTABLE_SUFFIX}" VERBATIM)
endif()
