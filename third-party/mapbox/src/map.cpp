#include "map.h"
#include "backend.h"

#include <mbgl/style/style.hpp>
#include <mbgl/util/default_styles.hpp>
#include <mbgl/gfx/backend_scope.hpp>

namespace terramach {

Map::Map(std::unique_ptr<Scheduler> scheduler_,
    std::unique_ptr<RendererFrontend> frontend_,
    const mbgl::MapOptions& options,
    const mbgl::ResourceOptions& resourceOptions)
    : scheduler(std::move(scheduler_)),
      frontend(std::move(frontend_)) {
    map = std::make_unique<mbgl::Map>(*frontend, *this, options, resourceOptions);
    mbgl::util::default_styles::DefaultStyle style = mbgl::util::default_styles::dark;
    map->getStyle().loadURL(style.url);
}

Map::~Map() {
    //Scheduler::SetCurrent(nullptr);
}

void Map::render() {
    auto backend = frontend->getRendererBackend();
    mbgl::gfx::BackendScope backendGuard { *backend };
    //Scheduler::SetCurrent(scheduler.get());
    frontend->render();
}

void Map::jumpTo(const mbgl::CameraOptions& camera) {
    map->jumpTo(camera);
}

void Map::easeTo(const mbgl::CameraOptions& camera, const mbgl::AnimationOptions& animation) {
    map->easeTo(camera, animation);
}

void Map::moveBy(const mbgl::ScreenCoordinate& coordinate, const mbgl::AnimationOptions* animation_) {
    map->moveBy(
        coordinate,
        animation_ == nullptr ? mbgl::AnimationOptions{} : *animation_
    );
}

void Map::scaleBy(double scale, const mbgl::ScreenCoordinate* anchor_, const mbgl::AnimationOptions* animation_) {
    map->scaleBy(
        scale,
        anchor_ == nullptr ? mbgl::nullopt : mbgl::optional<mbgl::ScreenCoordinate>(*anchor_),
        animation_ == nullptr ? mbgl::AnimationOptions{} : *animation_
    );
}

void Map::setSize(mbgl::Size size) {
    map->setSize(size);
}

mbgl::MapOptions Map::getMapOptions() const {
    return map->getMapOptions();
}

}

extern "C" void C_mbgl_ResourceOptions_withCachePath(mbgl::ResourceOptions* options, const char* path) {
    options->withCachePath(path);
}

extern "C" void C_mbgl_ResourceOptions_withAccessToken(mbgl::ResourceOptions* options, const char* token) {
    options->withAccessToken(token);
}

extern "C" terramach::Map* C_Map_new(
    terramach::Scheduler* scheduler,
    terramach::RendererFrontend* frontend,
    const mbgl::MapOptions& options,
    const mbgl::ResourceOptions& resourceOptions) {
    return new terramach::Map(
        std::unique_ptr<terramach::Scheduler>(scheduler),
        std::unique_ptr<terramach::RendererFrontend>(frontend),
        options,
        resourceOptions);
}
