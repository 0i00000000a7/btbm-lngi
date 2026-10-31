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
    'Back to live': '回到现在',
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
    'Reset time': '重置时间',
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
    'Extra columns are replaced by an ellipsis. Rendering cost grows with the column count, so a cap keeps the frame rate steady.':
        '超出的列会以省略号代替。渲染开销随列数增长，限制列数可以保持帧率稳定。',
    Close: '关闭',
    Auto: '自动',
};

/** Translate an English UI string. Unknown strings pass through unchanged. */
export function translate(lang: Lang, text: string): string {
    return lang === 'zh' ? (ZH[text] ?? text) : text;
}
