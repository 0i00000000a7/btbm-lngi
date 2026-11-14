<script setup lang="ts" vapor>
import { onMounted, ref, watchEffect } from 'vue';
import { writeOrdinal } from '../core/dom';

/**
 * Markup written into the page only when it changes.
 *
 * `v-html` assigns `innerHTML` on every re-render, and this page re-renders on
 * every animation frame, so a term that has not changed for hours was still
 * being parsed again sixty times a second — about a fifth of the main thread in
 * a phone profile. [`writeOrdinal`] skips the write when the string is the same
 * and keeps plain text out of the parser.
 *
 * The element is a `span`; the classes that used to sit on a `div` or a `p`
 * carry their own `display` in the stylesheet.
 */
const props = defineProps<{ html: string }>();

const el = ref<HTMLElement | null>(null);

onMounted(() => {
    if (el.value) writeOrdinal(el.value, props.html);
});

watchEffect(() => {
    if (el.value) writeOrdinal(el.value, props.html);
});
</script>

<template>
    <span ref="el" />
</template>
