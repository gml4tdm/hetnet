import collections


class ObjectIdMapping(collections.defaultdict):
    def __init__(self, *args, **kwargs):
        super().__init__(
            lambda: len(self),
            *args,
            **kwargs
        )
