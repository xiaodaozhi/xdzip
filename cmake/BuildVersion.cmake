find_program(XDZIP_GIT git)
set(XDZIP_REVISION "unknown")
set(XDZIP_DIRTY false)
if(XDZIP_GIT)
    execute_process(COMMAND "${XDZIP_GIT}" rev-parse --show-toplevel
        WORKING_DIRECTORY "${XDZIP_SOURCE_DIR}" OUTPUT_VARIABLE repository
        OUTPUT_STRIP_TRAILING_WHITESPACE ERROR_QUIET)
    get_filename_component(repository "${repository}" REALPATH)
    get_filename_component(source "${XDZIP_SOURCE_DIR}" REALPATH)
    if(repository STREQUAL source)
        execute_process(COMMAND "${XDZIP_GIT}" rev-parse --short=12 HEAD
            WORKING_DIRECTORY "${XDZIP_SOURCE_DIR}" OUTPUT_VARIABLE revision
            OUTPUT_STRIP_TRAILING_WHITESPACE ERROR_QUIET RESULT_VARIABLE status)
        if(status EQUAL 0)
            set(XDZIP_REVISION "${revision}")
            execute_process(COMMAND "${XDZIP_GIT}" status --porcelain --untracked-files=normal
                WORKING_DIRECTORY "${XDZIP_SOURCE_DIR}" OUTPUT_VARIABLE changes ERROR_QUIET)
            if(NOT changes STREQUAL "")
                set(XDZIP_DIRTY true)
            endif()
        endif()
    endif()
endif()
file(MAKE_DIRECTORY "${XDZIP_OUTPUT_DIR}")
configure_file("${XDZIP_SOURCE_DIR}/cmake/BuildVersion.h.in"
    "${XDZIP_OUTPUT_DIR}/BuildVersion.h" @ONLY)
