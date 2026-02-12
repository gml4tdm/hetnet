import typing

import torch

from .core import Graph, NodeRef
from ._node2vec import AbstractNode2Vec, Node2Vec as DefaultNode2Vec


class _ReporterWrapper:

    def __init__(self, reporter: typing.Callable[[int, str], None]):
        self.reporter = reporter
        self.last_message = None

    def __call__(self, progress: int, message: str):
        if self.last_message != message:
            self.reporter(progress, message)
            self.last_message = message


def _get_device(device_hint: str | None) -> torch.device:
    if device_hint is None:
        device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
    else:
        device = torch.device(device_hint)
    return device


def line(g: Graph, *,
         weighted: bool = True,
         embedding_size: int = 128,
         num_negative_samples: int = 5,
         learning_rate: float = 0.01,
         batch_size: int = 32,
         sparse: bool = False,
         epochs: int = 5,
         progress_reporter: typing.Callable[[int, str], None] = lambda x, y: None,
         num_threads: int = 1,
         device_hint: str | None = None:
    device = _get_device(device_hint)




def node2vec(g: Graph, *,
             node2vec_model: type[AbstractNode2Vec] = DefaultNode2Vec,
             # Random Walk & Sample Generation Settings
             weighted: bool = True,
             embedding_size: int = 128,
             walk_length: int = 20,
             context_size: int = 10,
             walks_per_node: int = 1,
             p: float = 1.0,
             q: float = 1.0,
             num_negative_samples: int = 1,
             negative_sampling_strategy: typing.Literal['unigram', 'uniform'] = 'uniform',
             unigram_walks_per_node: int = 5,
             # Training Settings
             learning_rate: float = 0.01,
             batch_size: int = 32,
             sparse: bool = False,
             epochs: int = 5,
             progress_reporter: typing.Callable[[int, str], None] = lambda x, y: None,
             num_threads: int = 1,
             num_workers: int = 1,
             device_hint: str | None = None,
             fast_walker: bool = False,
             n_workers: int = 1,
             node2vec_model: type[_node2vec.Node2Vec],
             **kwargs) -> tuple[torch.Tensor, dict[NodeRef, int], list[float]]:
    device = _get_device(device_hint)

    progress_reporter(0, 'Initialising model')
    model = node2vec_model(
        g,
        weighted=weighted,
        embedding_dim=embedding_size,
        walk_length=walk_length,
        context_size=context_size,
        walks_per_node=walks_per_node,
        p=p,
        q=q,
        num_negative_samples=num_negative_samples,
        negative_sampling_strategy=negative_sampling_strategy,
        unigram_walks_per_node=unigram_walks_per_node,
        sparse=sparse,
        fast_walker=fast_walker,
        n_workers=num_threads
    )
    model = model.to(device)

    progress_reporter(0, 'Model ready')

    loader = model.loader(batch_size=batch_size, shuffle=True, num_workers=num_workers)

    if sparse:
        optimiser = torch.optim.SparseAdam(model.parameters(), lr=learning_rate)
    else:
        optimiser = torch.optim.Adam(model.parameters(), lr=learning_rate)

    progress_reporter(0, 'Training')
    n_steps_per_epoch = len(loader)
    n_total_steps = epochs * n_steps_per_epoch
    n_steps = 0
    norm = len(loader) * (1 + model.num_negative_samples)
    losses = []
    for epoch in range(epochs):
        total_loss = 0
        for pos_rw, neg_rw, *args in loader:
            optimiser.zero_grad()
            loss = model.loss(pos_rw.to(device), neg_rw.to(device), *args)
            loss.backward()
            optimiser.step()
            total_loss += loss.item()
            n_steps += 1
            progress_reporter(
                n_steps * 100 // n_total_steps,
                f'Epoch {epoch} {(n_steps % n_steps_per_epoch) * 100 // n_steps_per_epoch:.2f}% done'
            )
        losses.append(total_loss / norm)

    progress_reporter(100, 'Done')

    embeddings = model.embedding.weight.detach().cpu()
    return embeddings, model.node_to_index_mapping, losses
