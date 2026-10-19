cmake_minimum_required(VERSION 3.24)
project(xdzip_toolchain_probe VERSION 0.1 LANGUAGES CXX)

# This isolated probe is not the desktop application or its build entry point.
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
find_package(Qt6 6.8 REQUIRED COMPONENTS Core Gui Qml Quick)

add_executable(xdzip-toolchain-probe main.cpp)
target_link_libraries(xdzip-toolchain-probe PRIVATE Qt6::Core Qt6::Gui Qt6::Qml Qt6::Quick)
if(MSVC)
    target_compile_options(xdzip-toolchain-probe PRIVATE /W4 /WX)
else()
    target_compile_options(xdzip-toolchain-probe PRIVATE -Wall -Wextra -Werror)
endif()

enable_testing()
add_test(NAME qt_window_lifetime COMMAND xdzip-toolchain-probe)
set_tests_properties(qt_window_lifetime PROPERTIES TIMEOUT 15)
