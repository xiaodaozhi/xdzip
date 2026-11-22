include_guard(GLOBAL)

# Declare the final copy as a file output so a missing copy can be restored.
function(xdzip_stage_file destination source)
    get_filename_component(directory "${destination}" DIRECTORY)
    add_custom_command(OUTPUT "${destination}"
        COMMAND "${CMAKE_COMMAND}" -E make_directory "${directory}"
        COMMAND "${CMAKE_COMMAND}" -E copy_if_different "${source}" "${destination}"
        DEPENDS "${source}" VERBATIM)
endfunction()
