You could make a wrapper type for .destroy() functions... 
Eh
nah

You could have a special wrapper for collections... So that anything drainable would be able to have their inner types destroyed...
Eh

Okay, so the plan is to have:
* Default engine core 
    * Really just a container for a set of supported stuff
* Backends
    * Really just a container again,
    * Also comes with functions for doing things in the container and interacting with those from the engine's container.
