<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, nextTick } from 'vue'
import * as echarts from 'echarts'

const props = defineProps<{
  data: number[]
  target: string
}>()

const chartRef = ref<HTMLElement | null>(null)
let chart: echarts.ECharts | null = null

// 初始化图表
function initChart() {
  if (!chartRef.value) return

  chart = echarts.init(chartRef.value)

  const option: echarts.EChartsOption = {
    grid: {
      top: 10,
      right: 20,
      bottom: 30,
      left: 50,
    },
    xAxis: {
      type: 'category',
      data: [],
      axisLabel: {
        formatter: (value: string) => {
          return value.split(' ')[1] || value
        },
        fontSize: 10,
        color: '#AEAEB2',
      },
      axisLine: {
        lineStyle: {
          color: 'rgba(0, 0, 0, 0.06)',
        },
      },
      axisTick: { show: false },
    },
    yAxis: {
      type: 'value',
      nameTextStyle: {
        fontSize: 11,
        color: '#AEAEB2',
      },
      axisLabel: {
        fontSize: 10,
        color: '#AEAEB2',
        formatter: '{value} ms',
      },
      splitLine: {
        lineStyle: {
          type: 'dashed',
          color: 'rgba(0, 0, 0, 0.04)',
        },
      },
    },
    series: [
      {
        name: '延迟',
        type: 'line',
        data: [],
        smooth: true,
        symbol: 'circle',
        symbolSize: 4,
        lineStyle: {
          width: 2,
          color: '#5E6AD2',
        },
        itemStyle: {
          color: '#5E6AD2',
        },
        areaStyle: {
          color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
            { offset: 0, color: 'rgba(94, 106, 210, 0.2)' },
            { offset: 1, color: 'rgba(94, 106, 210, 0.02)' },
          ]),
        },
      },
    ],
    tooltip: {
      trigger: 'axis',
      backgroundColor: 'rgba(255, 255, 255, 0.95)',
      borderColor: 'rgba(0, 0, 0, 0.06)',
      borderWidth: 1,
      textStyle: {
        color: '#1D1D1F',
        fontSize: 12,
      },
      formatter: (params: any) => {
        const data = params[0]
        return `${data.name}<br/>延迟: <strong>${data.value} ms</strong>`
      },
    },
  }

  chart.setOption(option)
}

// 更新图表数据
function updateChart() {
  if (!chart) return

  const now = new Date()
  const xData = props.data.map((_, i) => {
    const time = new Date(now.getTime() - (props.data.length - 1 - i) * 1000)
    return time.toLocaleTimeString('zh-CN', { hour12: false })
  })

  chart.setOption({
    xAxis: {
      data: xData,
    },
    series: [
      {
        data: props.data,
      },
    ],
  })
}

// 监听数据变化
watch(() => props.data, () => {
  nextTick(() => {
    updateChart()
  })
}, { deep: true })

// 监听窗口大小变化
function handleResize() {
  chart?.resize()
}

onMounted(() => {
  initChart()
  window.addEventListener('resize', handleResize)
})

onUnmounted(() => {
  window.removeEventListener('resize', handleResize)
  chart?.dispose()
})
</script>

<template>
  <div ref="chartRef" style="width: 100%; height: 100%;"></div>
</template>
