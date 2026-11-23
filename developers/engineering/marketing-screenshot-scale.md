# Marketing screenshot scale

English | [简体中文](../zh-CN/engineering/marketing-screenshot-scale.md)

Use **150%**, or **1.5 physical pixels per Qt logical pixel**, for exported Windows, macOS and Linux application screenshots intended for Store or marketing artwork. Keep the application at its normal logical dimensions and use the native display density for capture.

The current [README image](../../docs/images/main-window.png) is 1536 × 932 pixels. Its recorded Windows source was captured at 144 DPI, or 150%, with a 1536 × 933-pixel client area for the 1024 × 622 logical-pixel main window. The published image is one pixel shorter because its top edge was trimmed. The machine-readable target and reference dimensions are in [marketing-screenshot-scale.json](../../specs/capture/marketing-screenshot-scale.json).

Before capture, measure the application client area's actual logical dimensions and its native physical pixels, excluding operating-system frames and shadows. Confirm the measured ratio in both dimensions, allowing integer rounding at a pixel edge. The default main client is exported at 1536 × 933 physical pixels before edge processing. Other windows use their own measured logical sizes. Record the measurements and actual capture environment; an image's dimensions alone do not prove which window was captured or its scale.

On Windows the reference setting is 144 DPI. If the native density is higher, capture the normal window and record the measured density with `--capture-scale`. The exporter keeps the native PNG and downsamples the presentation image to 150%. For example, a 200% Retina capture of 2048 × 1244 becomes 1536 × 933. Never upscale. Record the source and export densities separately.

The [macOS and Linux recorder](platform-marketing.md) checks PNG integrity and operator-supplied identity; it does not independently measure Qt client geometry or prove the screenshot subject. The operator must supply and check this scale evidence. See the [Windows capture workflow](../market/windows/capture.md) for its DPI check and the [visual design standard](../../docs/visual/design-standard.md) for logical dimensions.
