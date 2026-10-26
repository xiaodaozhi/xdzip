# tools/native-codec/CMakeLists.txt
cmake_minimum_required(VERSION 3.24)
project(xdzip_native_codec LANGUAGES C CXX)
set(XDZIP_7Z_BACKEND rust CACHE INTERNAL "7z has one independent Rust product route" FORCE)
set(XDZIP_RAR_BACKEND rust CACHE INTERNAL "RAR has one independent Rust product route" FORCE)
set(XDZIP_CAB_BACKEND rust CACHE INTERNAL "CAB has one independent Rust product route" FORCE)
find_package(Threads REQUIRED)

# Every native source is project-owned stream/OS glue. Official archive
# handlers, algorithms, registries and filesystem adapters cannot re-enter.
set(native_glue_sources main.cpp seven_backend.cpp rar_backend.cpp readonly_backend.cpp
    codec_boundary.cpp platform_time.cpp platform_encoding.cpp)
add_executable(xdzip-native-codec ${native_glue_sources})
set(native_compiled_sources)
foreach(source IN LISTS native_glue_sources)
  get_filename_component(absolute "${source}" ABSOLUTE BASE_DIR "${CMAKE_CURRENT_SOURCE_DIR}")
  list(APPEND native_compiled_sources "${absolute}")
endforeach()
file(GENERATE OUTPUT "${CMAKE_CURRENT_BINARY_DIR}/native-codec-source-list.txt" CONTENT "$<JOIN:${native_compiled_sources},\n>\n")
target_compile_features(xdzip-native-codec PRIVATE cxx_std_17)
target_link_libraries(xdzip-native-codec PRIVATE Threads::Threads)
if(WIN32)
  target_link_libraries(xdzip-native-codec PRIVATE oleaut32 ole32 uuid advapi32 user32 shell32)
endif()

include("${CMAKE_CURRENT_SOURCE_DIR}/../../cmake/RustStaticLibrary.cmake")
xdzip_rust_library(xdzip-codec-rust xdzip-codec-bridge xdzip_codec_bridge "${CMAKE_CURRENT_SOURCE_DIR}/../..")
target_include_directories(xdzip-native-codec PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/xdzip-codec-bridge/include")
target_link_libraries(xdzip-native-codec PRIVATE xdzip-codec-rust)
