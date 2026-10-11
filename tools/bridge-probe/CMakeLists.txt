find_program(CARGO_EXECUTABLE cargo REQUIRED)
find_program(RUSTC_EXECUTABLE rustc REQUIRED)
find_package(Threads REQUIRED)
if(CMAKE_CROSSCOMPILING OR CMAKE_CONFIGURATION_TYPES OR CMAKE_OSX_ARCHITECTURES MATCHES ";")
    message(FATAL_ERROR "Diagnostic probe requires a native, single-configuration build")
endif()
execute_process(COMMAND "${RUSTC_EXECUTABLE}" -vV OUTPUT_VARIABLE rust_version
    WORKING_DIRECTORY "${PROJECT_SOURCE_DIR}" COMMAND_ERROR_IS_FATAL ANY)
string(REGEX MATCH "host: ([^\n\r]+)" host_match "${rust_version}")
set(rust_host "${CMAKE_MATCH_1}")
if(NOT rust_host)
    message(FATAL_ERROR "Cannot determine native Rust target")
endif()
set(rust_profile debug)
set(cargo_profile dev)
if(CMAKE_BUILD_TYPE STREQUAL "Release")
    set(rust_profile release)
    set(cargo_profile release)
endif()
set(rust_target_dir "${CMAKE_CURRENT_BINARY_DIR}/cargo")
if(WIN32)
    set(rust_archive "${rust_target_dir}/${rust_host}/${rust_profile}/xdzip_bridge_probe.lib")
else()
    set(rust_archive "${rust_target_dir}/${rust_host}/${rust_profile}/libxdzip_bridge_probe.a")
endif()
add_custom_command(OUTPUT "${rust_archive}"
    BYPRODUCTS "${rust_target_dir}/cxxbridge/xdzip-bridge-probe/src/lib.rs.h"
        "${rust_target_dir}/cxxbridge/rust/cxx.h"
    COMMAND "${CARGO_EXECUTABLE}" build --locked -p xdzip-bridge-probe
        --manifest-path "${PROJECT_SOURCE_DIR}/Cargo.toml"
        --target-dir "${rust_target_dir}" --target "${rust_host}" --profile "${cargo_profile}"
    WORKING_DIRECTORY "${PROJECT_SOURCE_DIR}"
    DEPENDS src/lib.rs src/transport.rs build.rs Cargo.toml
        "${PROJECT_SOURCE_DIR}/Cargo.toml" "${PROJECT_SOURCE_DIR}/Cargo.lock"
        "${PROJECT_SOURCE_DIR}/rust-toolchain.toml"
    VERBATIM)
add_custom_target(xdzip-probe-rust-build DEPENDS "${rust_archive}")
add_library(xdzip-probe-rust STATIC IMPORTED)
set_target_properties(xdzip-probe-rust PROPERTIES IMPORTED_LOCATION "${rust_archive}")
add_dependencies(xdzip-probe-rust xdzip-probe-rust-build)
add_executable(xdzip-bridge-probe smoke.cpp)
add_dependencies(xdzip-bridge-probe xdzip-probe-rust-build)
target_include_directories(xdzip-bridge-probe PRIVATE "${rust_target_dir}/cxxbridge")
target_link_libraries(xdzip-bridge-probe PRIVATE xdzip-probe-rust Threads::Threads ${CMAKE_DL_LIBS})
if(WIN32)
    target_link_libraries(xdzip-bridge-probe PRIVATE advapi32 bcrypt kernel32 ntdll userenv ws2_32)
elseif(APPLE)
    target_link_libraries(xdzip-bridge-probe PRIVATE iconv)
else()
    target_link_libraries(xdzip-bridge-probe PRIVATE m rt util)
endif()
if(MSVC)
    target_compile_options(xdzip-bridge-probe PRIVATE /W4 /WX /utf-8)
else()
    target_compile_options(xdzip-bridge-probe PRIVATE -Wall -Wextra -Werror)
endif()
add_test(NAME bridge_ownership_shutdown COMMAND xdzip-bridge-probe)
set_tests_properties(bridge_ownership_shutdown PROPERTIES TIMEOUT 15)

if(XDZIP_BUILD_QT_PROBE)
    find_package(Qt6 6.8 REQUIRED COMPONENTS Core)
    target_link_libraries(xdzip-bridge-probe PRIVATE Qt6::Core)
    target_compile_definitions(xdzip-bridge-probe PRIVATE XDZIP_PROBE_QT=1)
endif()
