pragma Singleton
import QtQuick

QtObject {
    // Extension hints are presentation only; they do not assert content or codec support.
    function fileIcon(name: string, directory: bool, large: bool): string {
        const suffix = name.toLowerCase().split(".").pop();
        let kind = "file-text";
        if (directory) kind = "folder";
        else if (suffix === "pdf") kind = "file-pdf";
        else if (suffix === "md") kind = "file-markdown";
        else if (["jpg", "jpeg", "png", "gif", "webp", "avif", "heic"].includes(suffix)) kind = "image-file";
        else if (["js", "ts", "rs", "cpp", "h", "json", "xml", "gpx", "py"].includes(suffix)) kind = "file-code";
        return "qrc:/xdzip/assets/" + (Theme.dark ? "dark/" : "light/") + kind + (large ? "-large" : "") + ".png";
    }
}
