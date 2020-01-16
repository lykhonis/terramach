#ifndef MAPBOX_FRONTEND_H
#define MAPBOX_FRONTEND_H

#include "backend.h"

#include <mbgl/renderer/renderer.hpp>
#include <mbgl/renderer/renderer_frontend.hpp>
#include <mbgl/util/run_loop.hpp>

namespace terramach {

#ifdef __cplusplus
extern "C" {
#endif

struct Frontend {
    void* info;
    float (*pixelRatio)(void* info);
    void (*invalidate)(void* info);
    void (*release)(void* info);
};

#ifdef __cplusplus
}
#endif

class RendererFrontend : public mbgl::RendererFrontend {
public:
    RendererFrontend(std::unique_ptr<RendererBackend>, const Frontend);
    ~RendererFrontend() override;

    void reset() override;
    void setObserver(mbgl::RendererObserver&) override;

    void update(std::shared_ptr<mbgl::UpdateParameters>) override;
    void render();

    RendererBackend* getRendererBackend();
    mbgl::Renderer* getRenderer();
private:
    const Frontend frontend;

    mbgl::util::RunLoop* mapRunLoop;
    std::unique_ptr<RendererBackend> backend;
    std::unique_ptr<mbgl::Renderer> renderer;
    std::shared_ptr<mbgl::UpdateParameters> updateParameters;
    std::unique_ptr<mbgl::RendererObserver> rendererObserver;
};

}

#endif //MAPBOX_FRONTEND_H
