/**
 * Tiny translation table: English is the source string, Chinese is looked up by
 * it. Notation names (BOCF, Veblen, 0-Y, DBMS, 1-Y/ω-Y, BTBMS) are the same in
 * both languages and are not listed here.
 */
export type Lang = 'en' | 'zh';

const ZH: Record<string, string> = {
    // status
    live: '实时',
    ahead: '超前',
    reviewing: '回溯中',
    // top bar
    'Milestones (dev)': '里程碑（开发）',
    Settings: '设置',
    // hero
    'Current ordinal': '当前序数',
    'to next ordinal': '至下一个序数',
    'until start': '后开始',
    // rewind
    Rewind: '回溯',
    'jump to a moment': '跳转到某一时刻',
    'Reset time': '重置时间',
    'Reset clock': '重置时钟',
    'Each slider is one digit; pick the granularities you want above.':
        '每个滑块是一位数字；先在上面选好需要的粒度。',
    ago: '前',
    '0 seconds': '0 秒',
    // rewind units
    millennia: '千年',
    centuries: '百年',
    decades: '十年',
    years: '年',
    days: '天',
    hours: '小时',
    minutes: '分钟',
    seconds: '秒',
    // dev controls
    dev: '开发',
    speed: '速度',
    'offset (s)': '偏移（秒）',
    '0× freezes time; negative runs it backwards': '0× 冻结时间，负数为时间倒流',
    // milestones
    Milestones: '里程碑',
    'Back to where the walk is': '回到行进位置',
    '↩ jump to 10 seconds before this landmark': '↩ 跳到该里程碑前 10 秒',
    'Every landmark reached. The sequence keeps going.': '所有里程碑均已到达。序列仍在继续。',
    'unreachable — the walk never gets there': '不可达——行进永远不会到达',
    'of this stretch': '这一段',
    'Click to follow the walk again': '点击以重新跟随行进',
    'Click to look at this landmark': '点击查看该里程碑',
    never: '永不',
    'after start': '开始后',
    // search
    Search: '查找',
    Progress: '进度',
    List: '列表',
    'e.g. (0)(1)(1)': '例如 (0)(1)(1)',
    'when is this ordinal reached?': '这个序数何时到达？',
    'Marked level': '标记层级',
    'Marked parent': '标记父项',
    "read each entry as its column's level": '把每一项读作它所在列的层级',
    'read each entry as its parent column': '把每一项读作它的父项列',
    'Type an ordinal to find its moment.': '输入序数，查出它的到达时刻。',
    'not a standard ordinal — the walk never reaches it': '不是标准序数——行进永远不会到达',
    'Type a date to look around it.': '输入日期，查看它附近的序数。',
    'no ordinal in this window': '这个区间里没有序数',
    day: '日',
    month: '月',
    year: '年',
    'the most remarkable ordinal in this day': '这一天里最显著的序数',
    'the most remarkable ordinal in this month': '这个月里最显著的序数',
    'the most remarkable ordinal in this year': '这一年里最显著的序数',
    // stats
    'Running at': '运行于',
    // panels
    Panels: '面板',
    '+ Add panel': '+ 添加面板',
    Expand: '展开',
    Collapse: '折叠',
    'Drag to resize': '拖动调整大小',
    Remove: '移除',
    // settings
    'Font family': '字体',
    'Font size': '字号',
    Language: '语言',
    'Default display mode': '默认显示模式',
    'Columns shown (0 = no limit)': '显示列数（0 = 不限制）',
    'Switch between elapsed and remaining time': '在已用时间和剩余时间之间切换',
    'Extra columns are replaced by an ellipsis. Rendering cost grows with the column count, so a cap keeps the frame rate steady.':
        '超出的列会以省略号代替。渲染开销随列数增长，限制列数可以保持帧率稳定。',
    Close: '关闭',
    Auto: '自动',
    // credits
    Credits: '致谢',
    'The coders': '开发者',
    'The notation': '记号',
    'The inspiration': '灵感来源',
    'the idea': '提出理念',
    'the definition and the expander': '定义与展开器',
};

/** Translate an English UI string. Unknown strings pass through unchanged. */
export function translate(lang: Lang, text: string): string {
    return lang === 'zh' ? (ZH[text] ?? text) : text;
}
