#ifndef MAPBOX_MAP_H
#define MAPBOX_MAP_H

#include <mbgl/map/map.hpp>

#include "frontend.h"
#include "scheduler.h"

namespace terramach {

class Map final: public mbgl::MapObserver {
public:
    Map(std::unique_ptr<Scheduler> scheduler,
        std::unique_ptr<RendererFrontend> frontend,
        const mbgl::MapOptions&,
        const mbgl::ResourceOptions&);
    ~Map();

    void render();

    void jumpTo(const mbgl::CameraOptions&);
    void easeTo(const mbgl::CameraOptions&, const mbgl::AnimationOptions&);
    void moveBy(const mbgl::ScreenCoordinate&, const mbgl::AnimationOptions*);
    void scaleBy(double scale, const mbgl::ScreenCoordinate* anchor, const mbgl::AnimationOptions* animation);

    void setSize(mbgl::Size);
    mbgl::MapOptions getMapOptions() const;
private:
    std::unique_ptr<Scheduler> scheduler;
    std::unique_ptr<RendererFrontend> frontend;
    std::unique_ptr<mbgl::Map> map;
};

}

#endif //MAPBOX_MAP_H
