# Marketing screenshot scale

English | [简体中文](../zh-CN/engineering/marketing-screenshot-scale.md)

Use **150%**, or **1.5 physical pixels per Qt logical pixel**, for native Windows, macOS and Linux application screenshots intended for Store or marketing artwork. This is a capture requirement, not a change to application layout or a requirement to set the same operating-system display preference on every platform.

The current [README image](../../docs/images/main-window.png) is 1536 × 932 pixels. Its recorded Windows source was captured at 144 DPI, or 150%, with a 1536 × 933-pixel client area for the 1024 × 622 logical-pixel main window. The published image is one pixel shorter because its top edge was trimmed. The machine-readable target and reference dimensions are in [marketing-screenshot-scale.json](../../specs/capture/marketing-screenshot-scale.json).

Before capture, measure the application client area's actual logical dimensions and its native physical pixels, excluding operating-system frames and shadows. Confirm the physical-to-logical ratio is 1.5 in both dimensions, allowing integer rounding at a pixel edge. For the default main client, that is 1536 × 933 physical pixels before edge processing. Other windows use their own measured logical sizes. Record the measurements and actual capture environment; an image's dimensions alone do not prove which window was captured or its scale.

On Windows the reference setting is 144 DPI. On macOS and Linux, select a capture environment that actually yields the same 1.5 ratio and verify the resulting client geometry. A Retina backing scale, desktop display setting or `QT_SCALE_FACTOR` value alone does not establish the final ratio; the effective Qt device-pixel ratio and native screenshot must agree. Do not resample an image to make a different capture scale look compliant, and do not encode DPR or physical screenshot dimensions in component sizes. If the target ratio cannot be achieved or measured, keep the capture unapproved and record the limitation.

The [macOS and Linux recorder](platform-marketing.md) checks PNG integrity and operator-supplied identity; it does not independently measure Qt client geometry or prove the screenshot subject. The operator must supply and check this scale evidence. See the [Windows capture workflow](../market/windows/capture.md) for its DPI check and the [visual design standard](../../docs/visual/design-standard.md) for logical dimensions.
