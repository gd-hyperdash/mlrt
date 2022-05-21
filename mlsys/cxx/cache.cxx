extern "C" void cxx_flush_cache(void* beg, void* end)
{
#if defined(__GNUC__)
    __builtin___clear_cache(
		reinterpret_cast<char*>(beg),
		reinterpret_cast<char*>(end));
#endif
}