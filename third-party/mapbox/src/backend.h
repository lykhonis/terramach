#ifndef MAPBOX_BACKEND_H
#define MAPBOX_BACKEND_H

#include <mbgl/gfx/renderable.hpp>
#include <mbgl/gl/renderer_backend.hpp>

namespace terramach {

#ifdef __cplusplus
extern "C" {
#endif

struct Backend {
    void* info;
    mbgl::gl::ProcAddress (*getExtensionFunctionPointer)(void* info, const char* name);
    mbgl::Size (*getFramebufferSize)(void* info);
    void (*makeCurrent)(void* info);
    void (*clearCurrent)(void* info);
    void (*presentCurrent)(void* info);
    void (*release)(void* info);
};

#ifdef __cplusplus
}
#endif

class RendererBackend final:
    public mbgl::gl::RendererBackend,
    public mbgl::gfx::Renderable {
public:
    RendererBackend(const Backend);
    ~RendererBackend() override;

    void swap();

    mbgl::gfx::RendererBackend& getRendererBackend() { return *this; }
    mbgl::Size getSize() const;
    void setSize(mbgl::Size);

    mbgl::gfx::Renderable& getDefaultRenderable() override { return *this; }

protected:
    void activate() override;
    void deactivate() override;

    mbgl::gl::ProcAddress getExtensionFunctionPointer(const char*) override;
    void updateAssumedState() override;

private:
    const Backend backend;
};

}

#endif //MAPBOX_BACKEND_H
