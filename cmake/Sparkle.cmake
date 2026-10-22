# cmake/Sparkle.cmake

# A build-time SDK stays external in ordinary native CI. Only the explicitly
# authorized deployment path embeds and signs the framework in a distributable.
find_package(Python3 REQUIRED COMPONENTS Interpreter)
set(XDZIP_SPARKLE_SDK "${CMAKE_BINARY_DIR}/_deps/sparkle-2.10.0" CACHE PATH "Pinned Sparkle SDK")
execute_process(COMMAND "${Python3_EXECUTABLE}" "${PROJECT_SOURCE_DIR}/scripts/prepare-sparkle.py"
    --output "${XDZIP_SPARKLE_SDK}" COMMAND_ERROR_IS_FATAL ANY OUTPUT_QUIET)
execute_process(COMMAND "${Python3_EXECUTABLE}" "${PROJECT_SOURCE_DIR}/scripts/prepare-sparkle.py"
    --public-key OUTPUT_VARIABLE XDZIP_SPARKLE_PUBLIC_KEY_BASE64
    OUTPUT_STRIP_TRAILING_WHITESPACE COMMAND_ERROR_IS_FATAL ANY)
find_library(XDZIP_SPARKLE_FRAMEWORK NAMES Sparkle PATHS "${XDZIP_SPARKLE_SDK}"
    NO_DEFAULT_PATH REQUIRED)
set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS
    "${PROJECT_SOURCE_DIR}/resources/sparkle-update-key.json"
    "${PROJECT_SOURCE_DIR}/scripts/sparkle-dependency.json")
