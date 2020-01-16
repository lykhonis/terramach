#include <mbgl/map/map.hpp>

extern "C" mbgl::CameraOptions* C_CameraOptions_new() {
    return new mbgl::CameraOptions();
}

extern "C" void C_CameraOptions_delete(mbgl::CameraOptions* instance) {
    delete instance;
}

extern "C" void C_CameraOptions_withCenter(mbgl::CameraOptions* instance, mbgl::LatLng& o) {
    instance->withCenter(o);
}

extern "C" void C_CameraOptions_withZoom(mbgl::CameraOptions* instance, double o) {
    instance->withZoom(o);
}

extern "C" mbgl::AnimationOptions* C_AnimationOptions_new() {
    return new mbgl::AnimationOptions();
}

extern "C" void C_AnimationOptions_delete(mbgl::AnimationOptions* instance) {
    delete instance;
}

extern "C" void C_AnimationOptions_setDuration(mbgl::AnimationOptions* instance, unsigned long duration) {
    instance->duration.emplace(mbgl::Milliseconds(duration));
}
