#include "scheduler.h"

#include <mbgl/util/run_loop.hpp>

namespace terramach {

Scheduler::Scheduler() {}

Scheduler::~Scheduler() {}

void Scheduler::schedule(std::function<void()> scheduled) {
}

}

extern "C" terramach::Scheduler* C_Scheduler_new() {
    return new terramach::Scheduler();
}
