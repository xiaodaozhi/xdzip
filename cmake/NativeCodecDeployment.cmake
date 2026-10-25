# cmake/NativeCodecDeployment.cmake

# Native codecs are a product dependency. Keep the worker beside each consumer,
# including the macOS bundle executable, so deployment needs no developer PATH.
foreach(consumer xdzip-browser xdzip-browser-tests xdzip-ui-preview xdzip-ui-tests)
    if(TARGET ${consumer})
        add_dependencies(${consumer} xdzip-native-codec)
        # A worker-only rebuild must refresh the consumer's copy of its wire protocol.
        set_property(TARGET ${consumer} APPEND PROPERTY LINK_DEPENDS "$<TARGET_FILE:xdzip-native-codec>")
        add_custom_command(TARGET ${consumer} POST_BUILD
            COMMAND "${CMAKE_COMMAND}" -E copy_if_different
                "$<TARGET_FILE:xdzip-native-codec>"
                "$<TARGET_FILE_DIR:${consumer}>/$<TARGET_FILE_NAME:xdzip-native-codec>"
            VERBATIM)
    endif()
endforeach()
