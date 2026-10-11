# Native-only cxx static libraries, built from the same locked workspace as the UI.
function(xdzip_rust_library target package stem)
    find_program(CARGO_EXECUTABLE cargo REQUIRED)
    find_program(RUSTC_EXECUTABLE rustc REQUIRED)
    find_package(Threads REQUIRED)
    if(CMAKE_CROSSCOMPILING OR CMAKE_CONFIGURATION_TYPES OR CMAKE_OSX_ARCHITECTURES MATCHES ";")
        message(FATAL_ERROR "Use a native single-configuration build for the current Rust bridge")
    endif()
    execute_process(COMMAND "${RUSTC_EXECUTABLE}" -vV OUTPUT_VARIABLE rust_version
        WORKING_DIRECTORY "${PROJECT_SOURCE_DIR}" COMMAND_ERROR_IS_FATAL ANY)
    string(REGEX MATCH "host: ([^\n\r]+)" host_match "${rust_version}")
    set(rust_host "${CMAKE_MATCH_1}")
    if(NOT rust_host)
        message(FATAL_ERROR "Unable to identify the native Rust target")
    endif()
    set(profile debug)
    set(cargo_profile dev)
    if(CMAKE_BUILD_TYPE STREQUAL "Release")
        set(profile release)
        set(cargo_profile release)
    endif()
    set(output "${CMAKE_CURRENT_BINARY_DIR}/rust")
    set(archive "${output}/${rust_host}/${profile}/${CMAKE_STATIC_LIBRARY_PREFIX}${stem}${CMAKE_STATIC_LIBRARY_SUFFIX}")
    set(cargo_environment)
    if(APPLE)
        if(NOT CMAKE_OSX_DEPLOYMENT_TARGET)
            message(FATAL_ERROR "Set CMAKE_OSX_DEPLOYMENT_TARGET for both Qt and Rust")
        endif()
        list(APPEND cargo_environment "MACOSX_DEPLOYMENT_TARGET=${CMAKE_OSX_DEPLOYMENT_TARGET}")
    endif()
    file(GLOB_RECURSE rust_sources CONFIGURE_DEPENDS
        "${PROJECT_SOURCE_DIR}/crates/*.rs" "${PROJECT_SOURCE_DIR}/crates/Cargo.toml"
        "${PROJECT_SOURCE_DIR}/crates/*/Cargo.toml")
    file(MAKE_DIRECTORY "${output}/cxxbridge")
    add_custom_command(OUTPUT "${archive}"
        BYPRODUCTS "${output}/cxxbridge/${package}/src/lib.rs.h" "${output}/cxxbridge/rust/cxx.h"
        COMMAND "${CMAKE_COMMAND}" -E env ${cargo_environment} "${CARGO_EXECUTABLE}" build
            --locked -p "${package}" --manifest-path "${PROJECT_SOURCE_DIR}/Cargo.toml"
            --target-dir "${output}" --target "${rust_host}" --profile "${cargo_profile}"
        DEPENDS ${rust_sources} "${PROJECT_SOURCE_DIR}/Cargo.lock" "${PROJECT_SOURCE_DIR}/Cargo.toml"
            "${PROJECT_SOURCE_DIR}/rust-toolchain.toml"
        WORKING_DIRECTORY "${PROJECT_SOURCE_DIR}" VERBATIM)
    add_custom_target(${target}-build DEPENDS "${archive}")
    add_library(${target} STATIC IMPORTED GLOBAL)
    set_target_properties(${target} PROPERTIES IMPORTED_LOCATION "${archive}"
        INTERFACE_INCLUDE_DIRECTORIES "${output}/cxxbridge")
    add_dependencies(${target} ${target}-build)
    target_link_libraries(${target} INTERFACE Threads::Threads ${CMAKE_DL_LIBS})
    if(WIN32)
        target_link_libraries(${target} INTERFACE advapi32 bcrypt kernel32 ntdll userenv ws2_32)
    elseif(APPLE)
        target_link_libraries(${target} INTERFACE iconv)
    else()
        target_link_libraries(${target} INTERFACE m rt util)
    endif()
endfunction()
