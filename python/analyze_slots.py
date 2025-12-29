import os
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
from pathlib import Path
from scipy import stats
from matplotlib.ticker import MaxNLocator

from plot_style import set_plot_style, get_colors_and_styles, format_axes, format_figure, format_axes_background

# 设置科研风格（论文风格）
set_plot_style('paper')


def get_project_root():
    """自动查找项目根目录，通过寻找 Cargo.toml 文件"""
    current_dir = os.path.dirname(os.path.abspath(__file__))
    while current_dir != os.path.dirname(current_dir):
        if os.path.exists(os.path.join(current_dir, 'Cargo.toml')):
            return current_dir
        current_dir = os.path.dirname(current_dir)
    return current_dir


def read_metrics_csv(consensus_type):
    """读取指定共识算法的 CSV 文件"""
    project_root = get_project_root()
    csv_file = os.path.join(project_root, f'metrics_slots_{consensus_type}.csv')
    
    if not os.path.exists(csv_file):
        print(f"警告: 找不到文件 {csv_file}")
        return None
    
    try:
        df = pd.read_csv(csv_file)
        if len(df) > 300:
            df = df.iloc[:300]
        print(f"成功读取 {consensus_type} 的数据: {len(df)} 条记录")
        return df
    except Exception as e:
        print(f"读取 {csv_file} 出错: {e}")
        return None




def create_gini_line_figure(dataframes_dict):
    """创建 Gini 系数折线图（论文风格多线对比）"""
    if not dataframes_dict:
        print("没有有效的数据")
        return
    
    fig, ax = plt.subplots(figsize=(10, 8))
    colors, linestyles, markers = get_colors_and_styles()
    
    for ct, df in dataframes_dict.items():
        if df is not None and len(df) > 0:
            gini = df['gini_coefficient'].values
            
            ax.plot(df.index, gini, 
                   label=f'{ct.upper()}',
                   color=colors.get(ct, '#000000'), 
                   linestyle=linestyles.get(ct, '-'),
                   marker=markers.get(ct), 
                   markevery=max(1, len(df) // 8),
                   alpha=0.9)
    
    format_axes(ax, xlabel='Slot', ylabel='Gini Coefficient', grid=True)
    ax.xaxis.set_major_locator(MaxNLocator(nbins=5))
    ax.yaxis.set_major_locator(MaxNLocator(nbins=6))
    ax.legend(fontsize=24, loc='upper right', frameon=True, fancybox=False, 
             edgecolor='black', framealpha=0.95, bbox_to_anchor=(1.0, 0.9))
    format_figure(fig)
    format_axes_background(ax)
    
    plt.tight_layout()
    
    project_root = get_project_root()
    output_file = os.path.join(project_root, 'figures', 'gini_coefficient.png')
    os.makedirs(os.path.dirname(output_file), exist_ok=True)
    plt.savefig(output_file, dpi=300, bbox_inches='tight', facecolor='white')
    print(f"✓ Gini系数图表已保存: {output_file}")
    plt.close()


def create_tps_line_figure(dataframes_dict):
    """创建 TPS (吞吐量) 对比图表（论文风格）"""
    if not dataframes_dict:
        print("没有有效的数据")
        return
    
    fig, ax = plt.subplots(figsize=(10, 8))
    colors, linestyles, markers = get_colors_and_styles()
    
    for ct, df in dataframes_dict.items():
        if df is not None and len(df) > 0:
            throughput = df['throughput'].values
            
            # 计算累计平均值
            cumulative_mean = np.cumsum(throughput) / np.arange(1, len(throughput) + 1)
            
            ax.plot(df.index, cumulative_mean, 
                    label=f'{ct.upper()}',
                    color=colors.get(ct), 
                    linestyle=linestyles.get(ct, '-'),
                    marker=markers.get(ct),
                    markevery=max(1, len(df) // 8),
                    alpha=0.9)
    
    format_axes(ax, xlabel='Slot', ylabel='Throughput (tx/s)', grid=True)
    ax.xaxis.set_major_locator(MaxNLocator(nbins=5))
    ax.yaxis.set_major_locator(MaxNLocator(nbins=5))
    ax.legend(fontsize=24, loc='best', frameon=True, fancybox=False, edgecolor='black', framealpha=0.95)
    format_figure(fig)
    format_axes_background(ax)
    
    plt.tight_layout()
    
    project_root = get_project_root()
    output_file = os.path.join(project_root, 'figures', 'tps_throughput.png')
    os.makedirs(os.path.dirname(output_file), exist_ok=True)
    plt.savefig(output_file, dpi=300, bbox_inches='tight', facecolor='white')
    print(f"✓ TPS吞吐量图表已保存: {output_file}")
    plt.close()


def create_path_length_line_figure(dataframes_dict):
    """创建交易平均路径长度对比图表（论文风格）"""
    if not dataframes_dict:
        print("没有有效的数据")
        return
    
    fig, ax = plt.subplots(figsize=(10, 8))
    colors, linestyles, markers = get_colors_and_styles()
    
    for ct, df in dataframes_dict.items():
        if df is not None and len(df) > 0:
            path_length = df['avg_path_length'].values
            
            # 计算累计平均值
            cumulative_mean = np.cumsum(path_length) / np.arange(1, len(path_length) + 1)
            
            ax.plot(df.index, cumulative_mean,
                    label=f'{ct.upper()}',
                    color=colors.get(ct),
                    linestyle=linestyles.get(ct, '-'),
                    marker=markers.get(ct),
                    markevery=max(1, len(df) // 8),
                    alpha=0.9)
    
    format_axes(ax, xlabel='Slot', ylabel='Average Path Length', grid=True)
    ax.xaxis.set_major_locator(MaxNLocator(nbins=5))
    ax.yaxis.set_major_locator(MaxNLocator(nbins=5))
    ax.legend(fontsize=24, loc='best', frameon=True, fancybox=False, edgecolor='black', framealpha=0.95)
    format_figure(fig)
    format_axes_background(ax)
    
    plt.tight_layout()
    
    project_root = get_project_root()
    output_file = os.path.join(project_root, 'figures', 'path_length.png')
    os.makedirs(os.path.dirname(output_file), exist_ok=True)
    plt.savefig(output_file, dpi=300, bbox_inches='tight', facecolor='white')
    print(f"✓ 路径长度图表已保存: {output_file}")
    plt.close()


def create_trend_figures(dataframes_dict):
    """创建趋势图表（随时间变化）- 已弃用"""
    pass


def create_tx_delay_line_figure(dataframes_dict):
    """创建平均交易打包延迟对比图表（论文风格）"""
    if not dataframes_dict:
        print("没有有效的数据")
        return
    
    fig, ax = plt.subplots(figsize=(10, 8))
    colors, linestyles, markers = get_colors_and_styles()
    
    for ct, df in dataframes_dict.items():
        if df is not None and len(df) > 0:
            # 检查是否有延迟列
            if 'avg_tx_delay_ms' in df.columns:
                tx_delay = df['avg_tx_delay_ms'].values
                
                # 计算累计平均值
                cumulative_mean = np.cumsum(tx_delay) / np.arange(1, len(tx_delay) + 1)
                
                ax.plot(df.index, cumulative_mean,
                        label=f'{ct.upper()}',
                        color=colors.get(ct),
                        linestyle=linestyles.get(ct, '-'),
                        marker=markers.get(ct),
                        markevery=max(1, len(df) // 8),
                        alpha=0.9)
    
    format_axes(ax, xlabel='Slot', ylabel='Transaction Packing Delay (s)', grid=True)
    ax.xaxis.set_major_locator(MaxNLocator(nbins=5))
    ax.yaxis.set_major_locator(MaxNLocator(nbins=5))
    ax.legend(fontsize=26, loc='best', frameon=True, fancybox=False, edgecolor='black', framealpha=0.95)
    format_figure(fig)
    format_axes_background(ax)
    
    plt.tight_layout()
    
    project_root = get_project_root()
    output_file = os.path.join(project_root, 'figures', 'tx_delay.png')
    os.makedirs(os.path.dirname(output_file), exist_ok=True)
    plt.savefig(output_file, dpi=300, bbox_inches='tight', facecolor='white')
    print(f"✓ 交易延迟图表已保存: {output_file}")
    plt.close()


def print_summary(dataframes_dict):
    """打印科研级统计摘要"""
    print("\n" + "="*90)
    print("共识算法性能统计报告 (Scientific Analysis Report)")
    print("="*90)
    
    for ct, df in dataframes_dict.items():
        if df is not None and len(df) > 0:
            print(f"\n【{ct.upper()} 共识算法】")
            print("-" * 90)
            
            # Gini 系数统计
            gini = df['gini_coefficient'].values
            print(f"  ├─ Gini系数 (公平性):")
            print(f"  │   ├─ 平均值 (μ):     {gini.mean():.6f}")
            print(f"  │   ├─ 标准差 (σ):     {gini.std():.6f}")
            print(f"  │   ├─ 中位数:          {np.median(gini):.6f}")
            print(f"  │   ├─ 范围:           [{gini.min():.6f}, {gini.max():.6f}]")
            print(f"  │   └─ 95% CI:         [{gini.mean() - 1.96*gini.std():.6f}, {gini.mean() + 1.96*gini.std():.6f}]")
            
            # TPS (吞吐量) 统计
            tps = df['throughput'].values
            print(f"  ├─ 吞吐量 TPS (tx/s):")
            print(f"  │   ├─ 平均值 (μ):     {tps.mean():.2f} tx/s")
            print(f"  │   ├─ 标准差 (σ):     {tps.std():.2f}")
            print(f"  │   ├─ 中位数:          {np.median(tps):.2f} tx/s")
            print(f"  │   ├─ 范围:           [{tps.min():.2f}, {tps.max():.2f}] tx/s")
            print(f"  │   └─ 变异系数 (CV):  {tps.std()/tps.mean():.4f}")
            
            # 路径长度统计
            path = df['avg_path_length'].values
            print(f"  ├─ 平均路径长度:")
            print(f"  │   ├─ 平均值 (μ):     {path.mean():.4f}")
            print(f"  │   ├─ 标准差 (σ):     {path.std():.4f}")
            print(f"  │   ├─ 中位数:          {np.median(path):.4f}")
            print(f"  │   └─ 范围:           [{path.min():.4f}, {path.max():.4f}]")
            
            # 延迟统计
            if 'tx_delay' in df.columns:
                delay = df['tx_delay'].values
                print(f"  ├─ 交易延迟 (ms):")
                print(f"  │   ├─ 平均值 (μ):     {delay.mean():.2f} ms")
                print(f"  │   ├─ 标准差 (σ):     {delay.std():.2f} ms")
                print(f"  │   ├─ 中位数:          {np.median(delay):.2f} ms")
                print(f"  │   └─ P95:            {np.percentile(delay, 95):.2f} ms")
            
            # 样本量信息
            print(f"  └─ 样本信息:")
            print(f"      ├─ 样本数 (N):      {len(df)}")
            print(f"      └─ 时间跨度:        {len(df)} slots")
    
    print("\n" + "="*90)
    print("说明:")
    print("  • μ (平均值): 样本均值")
    print("  • σ (标准差): 反映数据波动程度")
    print("  • CI (置信区间): 95% 置信度下的参数范围")
    print("  • CV (变异系数): σ/μ，用于衡量相对离散程度")
    print("="*90 + "\n")


if __name__ == '__main__':
    import sys
    
    print("\n" + "="*90)
    print("共识算法性能分析工具 v2.0 (Scientific Consensus Analysis Suite)")
    print("="*90)
    print("开始分析共识算法性能指标...\n")
    
    # 读取三种共识的数据
    consensus_types = ['pog', 'pos', 'pow']
    dataframes_dict = {}
    
    for ct in consensus_types:
        df = read_metrics_csv(ct)
        if df is not None:
            dataframes_dict[ct] = df
    
    # 打印统计摘要
    if dataframes_dict:
        print_summary(dataframes_dict)
    
    # 创建图表
    if dataframes_dict:
        print("\n生成学术风格论文图表...\n")
        print("[1/4] 生成Gini系数对比图表...")
        create_gini_line_figure(dataframes_dict)
        
        print("[2/4] 生成吞吐量(TPS)对比图表...")
        create_tps_line_figure(dataframes_dict)
        
        print("[3/4] 生成交易路径长度对比图表...")
        create_path_length_line_figure(dataframes_dict)
        
        print("[4/4] 生成交易打包延迟对比图表...")
        create_tx_delay_line_figure(dataframes_dict)
    
    print("\n" + "="*90)
    print("✓ 分析完成！")
    print("="*90)
    print("\n已生成的图表文件:")
    print("  📊 figures/gini_coefficient.png       - Gini系数对比分析")
    print("  📊 figures/tps_throughput.png         - 吞吐量(TPS)性能对比")
    print("  📊 figures/path_length.png            - 交易路径长度对比")
    print("  📊 figures/tx_delay.png               - 交易打包延迟对比")
    print("="*90 + "\n")
