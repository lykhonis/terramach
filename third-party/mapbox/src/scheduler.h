#ifndef MAPBOX_SCHEDULER_H
#define MAPBOX_SCHEDULER_H

#include <mbgl/actor/scheduler.hpp>

namespace terramach {

class Scheduler : public mbgl::Scheduler {
public:
    Scheduler();
    ~Scheduler() override;

    void schedule(std::function<void()> scheduled) override;
    mapbox::base::WeakPtr<mbgl::Scheduler> makeWeakPtr() override { return weakFactory.makeWeakPtr(); }
private:
    mapbox::base::WeakPtrFactory<mbgl::Scheduler> weakFactory{this};
};

}

#endif //MAPBOX_SCHEDULER_H
