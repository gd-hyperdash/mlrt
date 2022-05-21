#include <unordered_map>

using Map = std::unordered_map<std::uintptr_t, void*>;

extern "C" void* cxx_get_android_module_handle(
    std::uintptr_t const map,
    std::uintptr_t const handle)
{
    auto handlesMap = reinterpret_cast<Map const*>(map);
    auto it = handlesMap->find(handle);
    return it != handlesMap->end() ? it->second : nullptr;
}