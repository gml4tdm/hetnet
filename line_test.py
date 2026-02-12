import hetnet
import hetnet.embedding


builder = hetnet.GraphBuilder()
q1 = builder.add_node('Node')
q2 = builder.add_node('Node')
q3 = builder.add_node('Node')
q4 = builder.add_node('Node')
q5 = builder.add_node('Node')
q6 = builder.add_node('Node')
edges = [
    (q1, q2), (q2, q3), (q3, q1),
    (q4, q5), (q5, q6), (q6, q4),
    (q3, q4),
]
for fr, to in edges:
    builder.add_edge(fr, to, weight=1.0, kind='Edge')
    builder.add_edge(to, fr, weight=1.0, kind='Edge')

graph = builder.build()

embeddings, mapping = hetnet.embedding.line(
    graph,
    order='combined',
    weighted=True,
    embedding_size = 2,
    num_negative_samples=5,
    learning_rate=0.025,
    batch_size=1,
    sparse=True,
    epochs=20,
    progress_reporter=lambda x, y: print(f'[{x}]: {y}'),
    device_hint='cpu'
)

nodes = [q1, q2, q3, q4, q5, q6]
for i in nodes:
    for j in nodes:
        print(f'{i} -> {j}: {embeddings[mapping[i], :].dot(embeddings[mapping[j], :])}')

import matplotlib.pyplot as pyplot
from sklearn.manifold import TSNE

tsne = TSNE(n_components=2, init='pca', random_state=0, perplexity=2, metric='cosine')
embeddings = tsne.fit_transform(embeddings)

for lab, index in mapping.items():
    pyplot.scatter(embeddings[index, 0], embeddings[index, 1], label=str(lab))
pyplot.legend()
pyplot.show()
