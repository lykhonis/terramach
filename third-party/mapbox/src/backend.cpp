#include "backend.h"

#include <mbgl/gl/renderable_resource.hpp>

namespace terramach {

class RenderableResource final : public mbgl::gl::RenderableResource {
public:
    RenderableResource(RendererBackend& backend_) : backend(backend_) {}

    void bind() override {
        backend.setFramebufferBinding(0);
        backend.setViewport(0, 0, backend.getSize());
    }

    void swap() override {
        backend.swap();
    }

private:
    RendererBackend& backend;
};

RendererBackend::RendererBackend(const Backend backend_)
    : mbgl::gl::RendererBackend(mbgl::gfx::ContextMode::Unique),
      mbgl::gfx::Renderable((backend_.getFramebufferSize)(backend_.info), std::make_unique<RenderableResource>(*this)),
      backend(backend_) {}

RendererBackend::~RendererBackend() {
    if (backend.release) {
        (backend.release)(backend.info);
    }
}

void RendererBackend::activate() {
    (backend.makeCurrent)(backend.info);
}

void RendererBackend::deactivate() {
    (backend.clearCurrent)(backend.info);
}

mbgl::gl::ProcAddress RendererBackend::getExtensionFunctionPointer(const char* name) {
    return (backend.getExtensionFunctionPointer)(backend.info, name);
}

void RendererBackend::updateAssumedState() {
    assumeFramebufferBinding(0);
    setViewport(0, 0, size);
}

mbgl::Size RendererBackend::getSize() const {
    return size;
}

void RendererBackend::setSize(const mbgl::Size newSize) {
    size = newSize;
}

void RendererBackend::swap() {
    (backend.presentCurrent)(backend.info);
}

}

extern "C" terramach::RendererBackend* C_RendererBackend_new(const terramach::Backend backend) {
    return new terramach::RendererBackend(backend);
}
