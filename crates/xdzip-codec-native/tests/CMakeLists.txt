cmake_minimum_required(VERSION 3.24)
project(xdzip_auxiliary_protocol_fixture LANGUAGES CXX)
# This fixture compiles only project protocol code and an authored backend stub.
# It does not link or implement any archive codec.
add_executable(xdzip-auxiliary-protocol-worker
    ../../../tools/native-codec/main.cpp auxiliary_worker.cpp)
target_include_directories(xdzip-auxiliary-protocol-worker PRIVATE ../../../tools/native-codec)
target_compile_features(xdzip-auxiliary-protocol-worker PRIVATE cxx_std_17)
