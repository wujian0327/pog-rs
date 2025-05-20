import matplotlib
import numpy as np
import matplotlib.pyplot as plt
from matplotlib import cm
from matplotlib.patches import Rectangle


def block_probability(S, C, K=2, strategy=2):
    """
    计算出块概率的综合模型
    S: 真实权益占比 (0-1) 网格
    C: 网络贡献占比 (0-1) 网格
    设 总共1000个节点，共1000的真实质押和1000的网络贡献度
    """
    s_real = 1000 * S
    phi_s = np.where(S > 0.5, 0, 0.5 - S)
    s_v = s_real * (1 + K * C * phi_s)
    # 计算剩下的999个节点虚拟权益的总和
    if strategy == 1:
        # 1、我们这里使用均匀分配策略，平均分配策略得到的总虚拟权益是最小的
        s_real_one = 1000 * (1 - S) / 999
        phi_s = (1 - S) / 999
        phi_s = np.where(phi_s > 0.5, 0, 0.5 - phi_s)
        c = (1 - C) / 999
        s_v_one = s_real_one * (1 + K * c * phi_s)
        s_v_rest = s_v_one * 999
        s_v_total = s_v + s_v_rest
        result = s_v / s_v_total
        return result
    elif strategy == 2:
        # 2、假设剩下节点可以串谋，利用剩余资源构成出最大的总虚拟权益
        # 其实只用集中部分资源就行
        s_v_rest = [[]]
        sum = 0
        for i in range(1, 100):
            s_real_one = 1000 * (1 - S) / i
            phi_s = (1 - S) / i
            phi_s = np.where(phi_s > 0.5, 0, 0.5 - phi_s)
            c = (1 - C) / i
            s_v_one = s_real_one * (1 + K * c * phi_s)
            s_v_rest_i = s_v_one * i
            if np.sum(s_v_rest_i) > sum:
                sum = np.sum(s_v_rest_i)
                s_v_rest = s_v_rest_i
                print(i)
        s_v_total = s_v + s_v_rest
        result = s_v / s_v_total
        return result
    elif strategy == 3:
        s_v_rest = [[]]
        # 一个节点占
        sum = float('inf')
        for i in range(1, 50):
            s_real_one = 1000 * (1 - S) / i
            phi_s = (1 - S) / i
            phi_s = np.where(phi_s > 0.5, 0, 0.5 - phi_s)
            c = (1 - C) / i
            s_v_one = s_real_one * (1 + K * c * phi_s)
            s_v_rest_i = s_v_one * i
            if np.sum(s_v_rest_i) < sum:
                sum = np.sum(s_v_rest_i)
                s_v_rest = s_v_rest_i
                print(i)
        s_v_total = s_v + s_v_rest
        result = s_v / s_v_total
        return result


def print_selection(K=4):
    stake = np.linspace(0, 0.5, 100)
    contribution = np.linspace(0, 1, 100)
    S, C = np.meshgrid(stake, contribution)
    Z = block_probability(S, C, K, strategy=2)
    Z_max = block_probability(S, C, K, strategy=3)

    # 创建二维坐标网格
    fig = plt.figure(figsize=(10, 8))
    ax = fig.add_subplot(111, projection='3d')

    # ===== 关键改进部分 =====
    # 新颜色方案：紫色系 vs 黄绿色系
    ADV_CMAP = cm.Purples  # 对抗策略：紫色渐变
    COL_CMAP = cm.Greens  # 协作策略：绿色渐变
    EDGE_COLOR = 'k'  # 统一边缘线颜色
    ALPHA = 0.85  # 降低透明度增强对比

    # 绘制对抗策略曲面（带深色边缘）
    surf1 = ax.plot_surface(
        S, C, Z,
        cmap=ADV_CMAP,
        edgecolor=EDGE_COLOR,
        linewidth=0.8,
        alpha=ALPHA,
        rstride=2, cstride=2,
        antialiased=True
    )

    # 绘制协作策略曲面（带虚线边缘）
    surf2 = ax.plot_surface(
        S, C, Z_max,
        cmap=COL_CMAP,
        edgecolor='gray',
        linestyle=':',  # 虚线边缘
        linewidth=0.8,
        alpha=0.5,
        rstride=2, cstride=2,
        antialiased=True
    )

    # surf = ax.plot_surface(S, C, Z,
    #                        cmap='viridis',  # 颜色映射
    #                        edgecolor='k',  # 网格线颜色
    #                        linewidth=0.3,  # 网格线宽度
    #                        antialiased=True,  # 抗锯齿
    #                        rstride=5,  # 行步长（降低密度）
    #                        cstride=5)  # 列步长
    # surf2 = ax.plot_surface(S, C, Z_max,
    #                         cmap='viridis',  # 颜色映射
    #                         edgecolor='k',  # 网格线颜色
    #                         linewidth=0.3,  # 网格线宽度
    #                         antialiased=True,  # 抗锯齿
    #                         rstride=5,  # 行步长（降低密度）
    #                         cstride=5)  # 列步长

    # 设置视觉参数
    ax.view_init(elev=30, azim=150)  # 设置视角角度
    ax.set_xlabel('Real Stake Share (%)', labelpad=8, fontsize=12)
    ax.set_ylabel('Network Contribution Share (%)', labelpad=8, fontsize=12)
    ax.set_zlabel('Virtual Stake Share (%)', labelpad=8, fontsize=12)
    # ax.set_title(f'K={K}', fontsize=12, pad=0)

    ax.xaxis.set_major_formatter(matplotlib.ticker.PercentFormatter(1.0))
    ax.yaxis.set_major_formatter(matplotlib.ticker.PercentFormatter(1.0))
    ax.zaxis.set_major_formatter(matplotlib.ticker.PercentFormatter(1.0))

    legend_elements = [
        Rectangle((0, 0), 1, 1, fc='blue', alpha=0.6, label='Adversarial Strategy (Max)'),
        Rectangle((0, 0), 1, 1, fc='red', alpha=0.6, label='Collaborative Strategy (Min)')
    ]
    ax.legend(handles=legend_elements, loc='upper right')
    # 配置轴刻度百分比格式
    for axis in [ax.xaxis, ax.yaxis, ax.zaxis]:
        axis.set_major_formatter('{x:.0%}')

    # 添加颜色条
    # cbar = plt.colorbar(surf, shrink=0.5, aspect=10)
    # cbar.set_label('Probability Intensity', rotation=270, labelpad=20)

    # 添加参考平面
    # ax.plot_surface(S, C, np.zeros_like(Z), color='gray', alpha=0.2)  # 底部平面

    # 突出显示关键阈值线（示例）
    # ax.plot(stake, np.zeros_like(stake), block_probability(stake, 0),
    #         'r--', lw=2, label='Min Contribution')

    # ax.plot(stake, np.zeros_like(stake),
    #         'r--', lw=2, label='Min Contribution')

    plt.tight_layout()
    # plt.savefig(f'../figures/K{K}.png', dpi=600, bbox_inches=matplotlib.transforms.Bbox([[1.2, 0], [10, 7.5]]))
    plt.show()


if __name__ == '__main__':
    # for i in range(0, 8, 2):
    #     print_selection(i)
    print_selection(5)
