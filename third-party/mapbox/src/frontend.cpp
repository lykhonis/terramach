#include "frontend.h"

#include <mbgl/gfx/backend_scope.hpp>
#include <mbgl/actor/scheduler.hpp>
#include <mbgl/actor/actor.hpp>
#include <mbgl/renderer/renderer_observer.hpp>

namespace terramach {

class ForwardingRendererObserver : public mbgl::RendererObserver {
public:
    ForwardingRendererObserver(mbgl::util::RunLoop& mapRunLoop, mbgl::RendererObserver& delegate_)
            : mailbox(std::make_shared<mbgl::Mailbox>(mapRunLoop)),
              delegate(delegate_, mailbox) {
    }

    ~ForwardingRendererObserver() {
        mailbox->close();
    }

    void onInvalidate() override {
        delegate.invoke(&RendererObserver::onInvalidate);
    }

    void onResourceError(std::exception_ptr err) override {
        delegate.invoke(&RendererObserver::onResourceError, err);
    }

    void onWillStartRenderingMap() override {
        delegate.invoke(&RendererObserver::onWillStartRenderingMap);
    }

    void onWillStartRenderingFrame() override {
        delegate.invoke(&RendererObserver::onWillStartRenderingFrame);
    }

    void onDidFinishRenderingFrame(RenderMode mode, bool repaintNeeded, bool placementChanged) override {
        delegate.invoke(&RendererObserver::onDidFinishRenderingFrame, mode, repaintNeeded, placementChanged);
    }

    void onDidFinishRenderingMap() override {
        delegate.invoke(&RendererObserver::onDidFinishRenderingMap);
    }

    void onStyleImageMissing(const std::string& id, StyleImageMissingCallback done) override {
        delegate.invoke(&RendererObserver::onStyleImageMissing, id, done);
    }

    void onRemoveUnusedStyleImages(const std::vector<std::string>& ids) override {
        delegate.invoke(&RendererObserver::onRemoveUnusedStyleImages, ids);
    }

private:
    std::shared_ptr<mbgl::Mailbox> mailbox;
    mbgl::ActorRef<mbgl::RendererObserver> delegate;
};

RendererFrontend::RendererFrontend(std::unique_ptr<RendererBackend> backend_, const Frontend frontend_)
    : frontend(frontend_),
      mapRunLoop(mbgl::util::RunLoop::Get()),
      backend(std::move(backend_)),
      renderer(std::make_unique<mbgl::Renderer>(
          *backend,
          (frontend.pixelRatio)(frontend.info)
      )) {}

RendererFrontend::~RendererFrontend() {
    if (frontend.release) {
        (frontend.release)(frontend.info);
    }
}

void RendererFrontend::reset() {
    assert(renderer);
    renderer.reset();
}

void RendererFrontend::setObserver(mbgl::RendererObserver& observer) {
    assert(renderer);
    rendererObserver = std::make_unique<ForwardingRendererObserver>(*mapRunLoop, observer);
    renderer->setObserver(rendererObserver.get());
}

void RendererFrontend::update(std::shared_ptr<mbgl::UpdateParameters> params) {
    updateParameters = std::move(params);
    (frontend.invalidate)(frontend.info);
}

void RendererFrontend::render() {
    assert(renderer);
    if (!updateParameters) return;
    mbgl::gfx::BackendScope guard { *backend, mbgl::gfx::BackendScope::ScopeType::Implicit };
    auto updateParameters_ = updateParameters;
    renderer->render(updateParameters_);
    mapRunLoop->runOnce();
}

RendererBackend* RendererFrontend::getRendererBackend() {
    assert(backend);
    return backend.get();
}

mbgl::Renderer* RendererFrontend::getRenderer() {
    assert(renderer);
    return renderer.get();
}

}

extern "C" terramach::RendererFrontend* C_RendererFrontend_new(
    terramach::RendererBackend* backend,
    const terramach::Frontend frontend) {
    static mbgl::util::RunLoop mainRunLoop;
    return new terramach::RendererFrontend(
        std::unique_ptr<terramach::RendererBackend>(backend),
        frontend);
}
