#pragma once

class QDrag;

// A prepared native drag may start only while its original gesture is held.
bool nativeDragGestureHeld();
void executeNativeFileDrag(QDrag *drag);
