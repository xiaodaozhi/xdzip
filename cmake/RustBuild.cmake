include_guard(GLOBAL)

function(xdzip_rust_profile directory)
    if(CMAKE_BUILD_TYPE STREQUAL "RelWithDebInfo")
        set(profile relwithdebinfo)
        set(cargo relwithdebinfo)
    elseif(CMAKE_BUILD_TYPE STREQUAL "Release")
        set(profile release)
        set(cargo release)
    else()
        set(profile debug)
        set(cargo dev)
    endif()
    set(${directory} "${profile}" PARENT_SCOPE)
    if(ARGC GREATER 1)
        set(${ARGV1} "${cargo}" PARENT_SCOPE)
    endif()
endfunction()

function(xdzip_rust_inputs output workspace cargo host package)
    find_package(Python3 3.10 REQUIRED COMPONENTS Interpreter)
    set(input_script "${workspace}/scripts/rust-bridge-inputs.py")
    execute_process(COMMAND "${Python3_EXECUTABLE}" "${input_script}"
        --cargo "${cargo}" --target "${host}" --package "${package}"
        WORKING_DIRECTORY "${workspace}" OUTPUT_VARIABLE rust_crates
        OUTPUT_STRIP_TRAILING_WHITESPACE COMMAND_ERROR_IS_FATAL ANY)
    file(GLOB workspace_manifests CONFIGURE_DEPENDS
        "${workspace}/crates/*/Cargo.toml" "${workspace}/tools/*/Cargo.toml")
    set(workspace_inputs "${workspace}/Cargo.toml" "${workspace}/Cargo.lock"
        "${workspace}/rust-toolchain.toml" "${input_script}")
    set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS
        ${workspace_manifests} ${workspace_inputs})
    set(rust_sources ${workspace_inputs})
    foreach(crate IN LISTS rust_crates)
        file(GLOB_RECURSE crate_sources CONFIGURE_DEPENDS
            "${crate}/*.rs" "${crate}/*.h" "${crate}/*.hpp" "${crate}/data/*")
        list(APPEND rust_sources "${crate}/Cargo.toml" ${crate_sources})
        set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS "${crate}/Cargo.toml")
    endforeach()
    if("${workspace}/crates/xdzip-compression" IN_LIST rust_crates)
        list(APPEND rust_sources "${workspace}/third_party/brotli-rfc7932/dictionary.bin")
    endif()
    set(${output} "${rust_sources}" PARENT_SCOPE)
endfunction()
