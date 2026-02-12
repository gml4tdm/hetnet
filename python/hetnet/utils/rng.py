import heapq

import torch


class AliasSampler:

    @torch.no_grad()
    def __init__(self, weights: torch.Tensor):
        n = len(weights)
        total = torch.sum(weights)
        probability_table = weights / total * n

        alias_table = torch.arange(n)
        queue = _DoublePriorityQueue()
        for i, w in enumerate(probability_table.tolist()):
            queue.push(w, i)
        while (pair := queue.pop()) is not None:
            small, large = pair
            if small is not None and large is not None:
                w_l_new = min(1.0, max(0.0, large[0] - (1.0 - small[0])))
                probability_table[large[1]] = w_l_new
                alias_table[small[1]] = large[1]
                queue.push(w_l_new, large[1])
            elif small is not None or large is not None:
                i = small[1] if small is not None else large[1]  # type: ignore
                probability_table[i] = 1.0
                alias_table[i] = i

        self._probability_table = probability_table
        self._alias_table = alias_table
        self._value_table = torch.arange(n)
        self._n = n

    def sample(self, n: int):
        u = torch.rand(n) * self._n
        j = torch.floor(u).to(torch.int64)
        p = self._probability_table[j]
        out = self._value_table[j].clone()
        cond = u - j > p
        out[cond] = self._alias_table[j[cond]]
        return out



class _DoublePriorityQueue:

    def __init__(self):
        self._min_queue = []
        self._max_queue = []

    def push(self, w: float, i: int):
        if w < 1.0:
            heapq.heappush(self._min_queue, (w, i))
        else:
            heapq.heappush(self._max_queue, (-w, i))

    def pop(self):
        if self._min_queue and self._max_queue:
            small = heapq.heappop(self._min_queue)
            (w, i) = heapq.heappop(self._max_queue)
            return small, (-w, i)
        elif self._min_queue:
            return heapq.heappop(self._min_queue), None
        elif self._max_queue:
            (w, i) = heapq.heappop(self._max_queue)
            return None, (-w, i)
        else:
            return None
