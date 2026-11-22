# cmake/UpdateHelper.cmake

# Keep the replacement runner independent of Qt and, on Windows, the installed CRT.
if(WIN32 OR (UNIX AND NOT APPLE))
    include("${CMAKE_CURRENT_LIST_DIR}/StageFiles.cmake")
    find_program(XDZIP_UPDATE_CARGO cargo REQUIRED)
    find_program(XDZIP_UPDATE_RUSTC rustc REQUIRED)
    execute_process(COMMAND "${XDZIP_UPDATE_RUSTC}" -vV OUTPUT_VARIABLE update_rust_version
        COMMAND_ERROR_IS_FATAL ANY)
    string(REGEX MATCH "host: ([^\n\r]+)" update_host_match "${update_rust_version}")
    set(update_host "${CMAKE_MATCH_1}")
    if(NOT update_host)
        message(FATAL_ERROR "Unable to identify the native Rust target for the update helper")
    endif()
    xdzip_rust_profile(update_profile update_cargo_profile)
    set(update_target_dir "${CMAKE_CURRENT_BINARY_DIR}/update-helper-rust")
    set(update_binary "${update_target_dir}/${update_host}/${update_profile}/xdzip-update-helper${CMAKE_EXECUTABLE_SUFFIX}")
    set(update_environment)
    if(WIN32)
        list(APPEND update_environment "--unset=CARGO_ENCODED_RUSTFLAGS" "RUSTFLAGS=-C target-feature=+crt-static")
    endif()
    xdzip_rust_inputs(update_sources "${PROJECT_SOURCE_DIR}" "${XDZIP_UPDATE_CARGO}"
        "${update_host}" xdzip-update-helper)
    add_custom_command(OUTPUT "${update_binary}"
        COMMAND "${CMAKE_COMMAND}" -E env ${update_environment} "${XDZIP_UPDATE_CARGO}" build
            --locked -p xdzip-update-helper --manifest-path "${PROJECT_SOURCE_DIR}/Cargo.toml"
            --target-dir "${update_target_dir}" --target "${update_host}" --profile "${update_cargo_profile}"
        DEPENDS ${update_sources} "${PROJECT_SOURCE_DIR}/resources/update-trust.json"
        WORKING_DIRECTORY "${PROJECT_SOURCE_DIR}" VERBATIM)
    set(update_copy "${CMAKE_CURRENT_BINARY_DIR}/xdzip-update-helper${CMAKE_EXECUTABLE_SUFFIX}")
    xdzip_stage_file("${update_copy}" "${update_binary}")
    add_custom_target(xdzip-update-helper-build DEPENDS "${update_copy}")
    add_dependencies(xdzip-browser xdzip-update-helper-build)
endif()
