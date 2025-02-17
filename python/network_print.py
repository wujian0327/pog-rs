import json

import networkx as nx
import matplotlib.pyplot as plt
import numpy as np


def print_graph(json_file, output_path):
    """
    将graph.json文件转换为Matplotlib图表
    """

    with open(json_file, 'r') as file:
        data = json.load(file)
    # [ [   "17",  "9" ],[  "17",  "20" ]]
    G = nx.Graph()
    for edge in data:
        G.add_edge(edge[0], edge[1])

    # 高级布局配置
    pos = nx.spring_layout(
        G,
        k=1,  # 节点间距系数（值越大间距越大）
        iterations=1200,  # 布局迭代次数
        seed=42,  # 随机种子保持可重复性
        # scale=2.0,  # 图像缩放系数
        # threshold=1e-4  # 更严格的收敛阈值
    )

    # 节点可视化参数
    node_sizes = [500 + 30 * degree for _, degree in G.degree()]  # 按度数缩放节点大小
    node_colors = np.linspace(0.2, 1.0, num=len(node_sizes))  # 颜色渐变
    cmap = plt.cm.viridis  # 使用现代配色方案

    # 边样式配置
    edge_alpha = [0.3 + 0.7 * (np.random.rand()) for _ in G.edges()]  # 随机透明度增加层次感

    # 初始化画布
    plt.figure(figsize=(16, 12), dpi=300)  # 高分辨率大画布
    ax = plt.gca()

    # 绘制边（分批次绘制核心边和普通边）
    nx.draw_networkx_edges(
        G, pos, alpha=0.5, width=0.6, edge_color="#7F7F7F", ax=ax  # 基础透明度
    )

    # 绘制节点
    nodes = nx.draw_networkx_nodes(
        G,
        pos,
        node_size=node_sizes,
        node_color=node_colors,
        cmap=cmap,
        edgecolors="#444444",  # 节点边框颜色
        linewidths=0.8,
        alpha=0.95,
    )

    # 标签策略
    important_nodes = [n for n, d in G.degree() if d > 5]  # 只标记高度数节点
    labels = {n: str(n) for n in important_nodes}
    nx.draw_networkx_labels(
        G,
        pos,
        labels=labels,
        font_size=8,
        font_family="sans-serif",
        font_color="#333333",
        alpha=0.9,
    )

    # 添加装饰元素
    plt.colorbar(nodes, label="Node Centrality", shrink=0.8)  # 颜色条说明
    ax.set_title("Complex Network Visualization (100 Nodes)", fontsize=14, pad=20)

    # 优化画布细节
    ax.margins(0.02)
    plt.axis("off")
    plt.tight_layout()

    # 保存输出
    plt.savefig(output_path, bbox_inches="tight", transparent=False)
    plt.close()


if __name__ == '__main__':
    # 示例使用
    print_graph(
        '../graph.json',
        output_path='../figures/graph.png',
    )
