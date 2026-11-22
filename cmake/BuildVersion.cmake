set(XDZIP_REVISION "unknown")
set(XDZIP_DIRTY false)
if(NOT DEFINED XDZIP_GIT OR XDZIP_GIT)
    if(NOT XDZIP_IDENTITY_PYTHON)
        find_program(XDZIP_IDENTITY_PYTHON NAMES python3 python REQUIRED)
    endif()
    execute_process(COMMAND "${XDZIP_IDENTITY_PYTHON}" -B
        "${CMAKE_CURRENT_LIST_DIR}/../scripts/build_identity.py" "${XDZIP_SOURCE_DIR}"
        OUTPUT_VARIABLE identity COMMAND_ERROR_IS_FATAL ANY)
    string(JSON XDZIP_REVISION GET "${identity}" revision)
    string(JSON dirty GET "${identity}" dirty)
    if(dirty)
        set(XDZIP_DIRTY true)
    endif()
endif()
file(MAKE_DIRECTORY "${XDZIP_OUTPUT_DIR}")
configure_file("${XDZIP_SOURCE_DIR}/cmake/BuildVersion.h.in"
    "${XDZIP_OUTPUT_DIR}/BuildVersion.h" @ONLY)
