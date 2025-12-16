import numpy as np
import matplotlib.pyplot as plt
import matplotlib as mpl

# 设置matplotlib的样式
mpl.rcParams['font.sans-serif'] = ['DejaVu Sans']
mpl.rcParams['axes.unicode_minus'] = False

def generate_block_production_data():
    """
    生成模拟的不同掉线概率下各共识算法的出块成功率数据
    掉线概率范围: 0% - 50%（每个节点在每个时刻有X%的概率掉线）
    """
    offline_rates = np.array([0, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50])
    
    pog_success_rates = np.array([1, 0.99, 0.98, 0.97, 0.96, 0.92, 0.88, 0.84, 0.80, 0.78, 0.77]) * 100
    
    pos_success_rates = np.array([1, 0.99, 0.97, 0.92, 0.88, 0.85, 0.80, 0.75,115/(115+38), 0.66, 0.61]) * 100
    
    
    return offline_rates, pog_success_rates, pos_success_rates

def create_block_production_rate_figure():
    """
    绘制不同掉线率下的出块成功率对比图
    """
    # 获取数据
    offline_rates, pog_rates, pos_rates = generate_block_production_data()
    
    # 创建图表
    fig, ax = plt.subplots(figsize=(10, 7))
    
    # 绘制三条曲线
    ax.plot(offline_rates, pog_rates, 
            color='#1f77b4', linewidth=2.5, linestyle='-', 
            marker='o', markersize=8, label='PoG', zorder=3)
    
    ax.plot(offline_rates, pos_rates, 
            color='#2ca02c', linewidth=2.5, linestyle='--', 
            marker='s', markersize=8, label='PoS', zorder=3)

    
    # 设置网格
    ax.grid(True, linestyle='--', alpha=0.5, linewidth=0.7, zorder=1)
    ax.set_axisbelow(True)
    
    # 设置轴样式
    ax.spines['top'].set_linewidth(1.5)
    ax.spines['bottom'].set_linewidth(1.5)
    ax.spines['left'].set_linewidth(1.5)
    ax.spines['right'].set_linewidth(1.5)
    
    # 设置标签字体大小
    ax.set_xlabel('Node Offline Probability (%)', fontsize=16, fontweight='normal')
    ax.set_ylabel('Block Production Success Rate (%)', fontsize=16, fontweight='normal')
    ax.set_title('Block Production Success Rate vs. Node Offline Probability', fontsize=17, fontweight='bold', pad=20)
    
    # 设置坐标轴刻度字体大小
    ax.tick_params(axis='x', labelsize=12)
    ax.tick_params(axis='y', labelsize=12)
    
    # 设置坐标轴范围
    ax.set_xlim(-2, 52)
    ax.set_ylim(30, 110)
    
    # 设置图例
    ax.legend(fontsize=18, loc='upper right', frameon=True, fancybox=False, edgecolor='black')
    
    plt.tight_layout()
    plt.savefig('block_production_rate.png', dpi=300, bbox_inches='tight')
    print("[1/1] Block production rate figure generated successfully!")
    print("      Figure saved as: block_production_rate.png")
    plt.show()

if __name__ == '__main__':
    print("\n========== Block Production Rate Analysis ==========\n")
    create_block_production_rate_figure()
    print("\n====================================================\n")
