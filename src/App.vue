<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ElMessage, ElMessageBox } from 'element-plus'
import PingChart from './components/PingChart.vue'

// 接口定义
interface PingResult {
  target: string
  ip: string
  status: string
  latency: number
  avg_latency: number
  max_latency: number
  min_latency: number
  loss_rate: number
  sent: number
  received: number
  timestamp: number
}

interface PingConfig {
  targets: string[]
  packet_size: number
  interval: number
  timeout: number
  mode: string
  count: number
}

// 状态
const targetInput = ref(`114.114.114.114
baidu.com
8.8.8.8`)
const packetSize = ref(32)
const interval = ref(1000)
const timeout = ref(3000)
const pingMode = ref('continuous')
const pingCount = ref(10)
const isRunning = ref(false)
const pingResults = ref<Map<string, PingResult>>(new Map())
const selectedTarget = ref('')
const chartData = ref<Map<string, number[]>>(new Map())
const showGroupDialog = ref(false)

// 主题切换
const isDark = ref(localStorage.getItem('theme') === 'dark' ||
  (!localStorage.getItem('theme') && window.matchMedia('(prefers-color-scheme: dark)').matches))

function toggleTheme() {
  isDark.value = !isDark.value
  localStorage.setItem('theme', isDark.value ? 'dark' : 'light')
  document.documentElement.classList.toggle('dark', isDark.value)
}

// 初始化主题
document.documentElement.classList.toggle('dark', isDark.value)

// 计算属性
const tableData = computed(() => {
  return Array.from(pingResults.value.values())
})

const totalTargets = computed(() => pingResults.value.size)
const activeTargets = computed(() => {
  return Array.from(pingResults.value.values()).filter(r => r.status === '测试中').length
})
const avgLatency = computed(() => {
  const results = Array.from(pingResults.value.values())
  if (results.length === 0) return 0
  const sum = results.reduce((acc, r) => acc + r.avg_latency, 0)
  return Math.round(sum / results.length)
})

// 监听Tauri事件
let unlisten: (() => void) | null = null

async function setupEventListener() {
  unlisten = await listen<PingResult>('ping-result', (event) => {
    const result = event.payload
    pingResults.value.set(result.target, result)

    // 更新图表数据
    const data = chartData.value.get(result.target) || []
    data.push(result.latency)
    // 限制数据点数量，防止卡顿
    if (data.length > 100) {
      data.shift()
    }
    chartData.value.set(result.target, data)
  })
}

// 解析目标输入
function parseTargets(): string[] {
  return targetInput.value
    .split('\n')
    .map(line => line.trim())
    .filter(line => line.length > 0)
    .filter((value, index, self) => self.indexOf(value) === index) // 去重
}

// 开始Ping
async function startPing() {
  const targets = parseTargets()
  if (targets.length === 0) {
    ElMessage.warning('请输入至少一个Ping目标')
    return
  }

  // 清空之前的结果
  pingResults.value.clear()
  chartData.value.clear()

  const config: PingConfig = {
    targets,
    packet_size: packetSize.value,
    interval: interval.value,
    timeout: timeout.value,
    mode: pingMode.value,
    count: pingCount.value,
  }

  try {
    isRunning.value = true
    await setupEventListener()
    await invoke('start_ping', { config })
  } catch (error) {
    ElMessage.error(`启动Ping失败: ${error}`)
    isRunning.value = false
  }
}

// 停止Ping
async function stopPing() {
  try {
    await invoke('stop_ping')
    isRunning.value = false
    if (unlisten) {
      unlisten()
      unlisten = null
    }
    ElMessage.success('已停止所有Ping任务')
  } catch (error) {
    ElMessage.error(`停止Ping失败: ${error}`)
  }
}

// 清空全部
function clearAll() {
  if (isRunning.value) {
    ElMessageBox.confirm('正在测试中，确定要清空全部吗？', '提示', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    }).then(() => {
      stopPing()
      pingResults.value.clear()
      chartData.value.clear()
      selectedTarget.value = ''
    }).catch(() => {})
  } else {
    pingResults.value.clear()
    chartData.value.clear()
    selectedTarget.value = ''
  }
}

// 选中行
function handleRowClick(row: PingResult) {
  selectedTarget.value = row.target
}

// 导出CSV
async function exportCSV() {
  const results = Array.from(pingResults.value.values())
  if (results.length === 0) {
    ElMessage.warning('没有数据可导出')
    return
  }

  const headers = ['目标地址', '解析IP', '状态', '最新延迟', '平均延迟', '最大延迟', '最小延迟', '丢包率', '发送包数', '接收包数']
  const rows = results.map(r => [
    r.target,
    r.ip,
    r.status,
    r.latency,
    r.avg_latency,
    r.max_latency,
    r.min_latency,
    `${r.loss_rate}%`,
    r.sent,
    r.received,
  ])

  const csvContent = [
    headers.join(','),
    ...rows.map(row => row.join(',')),
  ].join('\n')

  // 添加BOM头，解决Excel中文乱码
  const bom = '﻿'
  const blob = new Blob([bom + csvContent], { type: 'text/csv;charset=utf-8;' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = `ping_report_${new Date().toISOString().slice(0, 10)}.csv`
  link.click()
  URL.revokeObjectURL(url)
  ElMessage.success('导出成功')
}

// 格式化延迟显示
function formatLatency(latency: number): string {
  if (latency === 0) return '-'
  return `${latency} ms`
}

// 获取状态样式类
function getStatusClass(status: string): string {
  if (status === '成功' || status === '测试中') return 'status-success'
  if (status === '失败' || status === '超时') return 'status-fail'
  return 'status-testing'
}

// 获取当前选中IP的图表数据
const currentChartData = computed(() => {
  if (!selectedTarget.value) return []
  return chartData.value.get(selectedTarget.value) || []
})
</script>

<template>
  <div class="app-container">
    <!-- 左侧配置区 -->
    <div class="left-panel">
      <div class="panel-header">
        <div class="logo">快</div>
        <div>
          <h1>快Ping</h1>
          <div class="subtitle">轻量高效的网络测试工具</div>
        </div>
      </div>

      <!-- 目标输入 -->
      <div class="config-section">
        <h3>🎯 Ping目标</h3>
        <div class="target-input">
          <el-input
            v-model="targetInput"
            type="textarea"
            :rows="6"
            placeholder="每行一个IP或域名，例如：&#10;114.114.114.114&#10;baidu.com&#10;8.8.8.8"
            :disabled="isRunning"
          />
        </div>
      </div>

      <!-- 参数配置 -->
      <div class="config-section">
        <h3>⚙️ 参数配置</h3>
        <div class="param-row">
          <div class="param-item">
            <label>数据包大小 (bytes)</label>
            <el-input-number v-model="packetSize" :min="1" :max="65500" :step="1" size="small" />
          </div>
          <div class="param-item">
            <label>发送间隔 (ms)</label>
            <el-input-number v-model="interval" :min="100" :max="10000" :step="100" size="small" />
          </div>
        </div>
        <div class="param-row">
          <div class="param-item">
            <label>超时时间 (ms)</label>
            <el-input-number v-model="timeout" :min="500" :max="30000" :step="500" size="small" />
          </div>
          <div class="param-item">
            <label>Ping模式</label>
            <el-select v-model="pingMode" size="small">
              <el-option label="持续Ping" value="continuous" />
              <el-option label="指定次数" value="count" />
            </el-select>
          </div>
        </div>
        <div class="param-row" v-if="pingMode === 'count'">
          <div class="param-item">
            <label>Ping次数</label>
            <el-input-number v-model="pingCount" :min="1" :max="10000" :step="1" size="small" />
          </div>
        </div>
      </div>

      <!-- 按钮组 -->
      <div class="button-group">
        <el-button
          type="primary"
          @click="startPing"
          :disabled="isRunning"
          :icon="isRunning ? 'Loading' : 'VideoPlay'"
        >
          {{ isRunning ? '测试中...' : '开始测试' }}
        </el-button>
        <el-button
          type="danger"
          @click="stopPing"
          :disabled="!isRunning"
          icon="VideoPause"
        >
          停止
        </el-button>
      </div>

      <div class="button-group" style="padding-top: 0;">
        <el-button @click="clearAll" icon="Delete">清空全部</el-button>
        <el-button @click="exportCSV" icon="Download" type="success">导出CSV</el-button>
      </div>

      <!-- 加群链接 -->
      <div class="group-link" @click="showGroupDialog = true">
        💬 加入软件沟通群
      </div>
    </div>

    <!-- 右侧结果区 -->
    <div class="right-panel">
      <!-- 统计栏 -->
      <div class="result-header">
        <h2>测试结果</h2>
        <div class="header-actions">
          <div class="theme-toggle" @click="toggleTheme" :title="isDark ? '切换到亮色模式' : '切换到暗色模式'">
            {{ isDark ? '☀️' : '🌙' }}
          </div>
          <div class="stats-bar">
          <div class="stat-card">
            <div class="stat-label">目标数</div>
            <div class="stat-value">{{ totalTargets }}</div>
          </div>
          <div class="stat-card">
            <div class="stat-label">活跃</div>
            <div class="stat-value">{{ activeTargets }}</div>
          </div>
          <div class="stat-card">
            <div class="stat-label">平均延迟</div>
            <div class="stat-value">{{ avgLatency }}<span class="stat-unit">ms</span></div>
          </div>
        </div>
        </div>
      </div>

      <!-- 结果表格 -->
      <div class="table-container">
        <el-table
          :data="tableData"
          class="ping-table"
          highlight-current-row
          @row-click="handleRowClick"
          :row-class-name="({ row }: any) => row.target === selectedTarget ? 'current-row' : ''"
          size="small"
          :max-height="400"
        >
          <el-table-column prop="target" label="目标地址" min-width="120" show-overflow-tooltip />
          <el-table-column prop="ip" label="解析IP" min-width="120" show-overflow-tooltip />
          <el-table-column prop="status" label="状态" width="80">
            <template #default="{ row }">
              <span :class="getStatusClass(row.status)">{{ row.status }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="latency" label="最新延迟" width="100">
            <template #default="{ row }">
              {{ formatLatency(row.latency) }}
            </template>
          </el-table-column>
          <el-table-column prop="avg_latency" label="平均延迟" width="100">
            <template #default="{ row }">
              {{ formatLatency(row.avg_latency) }}
            </template>
          </el-table-column>
          <el-table-column prop="max_latency" label="最大延迟" width="100">
            <template #default="{ row }">
              {{ formatLatency(row.max_latency) }}
            </template>
          </el-table-column>
          <el-table-column prop="min_latency" label="最小延迟" width="100">
            <template #default="{ row }">
              {{ formatLatency(row.min_latency) }}
            </template>
          </el-table-column>
          <el-table-column prop="loss_rate" label="丢包率" width="80">
            <template #default="{ row }">
              <span :style="{ color: row.loss_rate > 10 ? '#f56c6c' : '#67c23a' }">
                {{ row.loss_rate }}%
              </span>
            </template>
          </el-table-column>
          <el-table-column prop="sent" label="发送" width="70" />
          <el-table-column prop="received" label="接收" width="70" />
        </el-table>
      </div>

      <!-- 图表区域 -->
      <div class="chart-container">
        <div class="chart-header">
          <h3>📈 延迟曲线图 {{ selectedTarget ? `- ${selectedTarget}` : '(请选中表格中的一行)' }}</h3>
        </div>
        <div class="chart-wrapper">
          <PingChart :data="currentChartData" :target="selectedTarget" />
        </div>
      </div>
    </div>

    <!-- 加群弹框 -->
    <el-dialog
      v-model="showGroupDialog"
      title="加入软件沟通群"
      width="360px"
      :show-close="true"
      center
    >
      <div class="group-dialog-content">
        <p>本软件由公众号 <strong>网络技术联盟站</strong> 开发</p>
        <p>关注后回复 <strong>加群</strong> 二字</p>
        <img src="/wechat.jpg" alt="公众号二维码" class="qrcode-img" />
        <p class="qrcode-tip">长按识别二维码关注公众号</p>
      </div>
    </el-dialog>
  </div>
</template>

<style scoped>
</style>
