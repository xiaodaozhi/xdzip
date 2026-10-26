cmake_minimum_required(VERSION 3.24)
project(xdzip_auxiliary_protocol_fixture LANGUAGES CXX)
# This fixture compiles only project protocol code and an authored backend stub.
# It does not link or implement any archive codec.
add_executable(xdzip-auxiliary-protocol-worker
    ../../../tools/native-codec/main.cpp auxiliary_worker.cpp)
target_include_directories(xdzip-auxiliary-protocol-worker PRIVATE ../../../tools/native-codec)
target_compile_features(xdzip-auxiliary-protocol-worker PRIVATE cxx_std_17)

# This opt-in probe uses the already locked CXX runtime and the worker's real
# generated header. It calls only project-owned OS character mapping glue.
if(DEFINED XDZIP_CODEC_BRIDGE_INCLUDE AND DEFINED XDZIP_CODEC_BRIDGE_ARCHIVE)
  add_executable(xdzip-native-codepage-probe codepage_probe.cpp
      ../../../tools/native-codec/platform_encoding.cpp)
  target_include_directories(xdzip-native-codepage-probe PRIVATE
      "${XDZIP_CODEC_BRIDGE_INCLUDE}" ../../../crates/xdzip-codec-bridge/include)
  target_compile_features(xdzip-native-codepage-probe PRIVATE cxx_std_17)
  find_package(Threads REQUIRED)
  target_link_libraries(xdzip-native-codepage-probe PRIVATE
      "${XDZIP_CODEC_BRIDGE_ARCHIVE}" Threads::Threads ${CMAKE_DL_LIBS})
  if(WIN32)
    target_link_libraries(xdzip-native-codepage-probe PRIVATE
        user32 advapi32 bcrypt kernel32 ntdll userenv ws2_32)
  elseif(APPLE)
    target_link_libraries(xdzip-native-codepage-probe PRIVATE iconv)
  else()
    target_link_libraries(xdzip-native-codepage-probe PRIVATE m rt util)
  endif()
endif()
