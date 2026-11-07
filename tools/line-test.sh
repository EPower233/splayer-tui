#!/usr/bin/env bash
# 检查当前终端能不能画出 TUI 用的分隔线。用法：bash line-test.sh
# 第 5 节会向终端查询光标位置来测字符宽度；输出不是终端（或设置 NOQUERY=1）时跳过。

E=$'\033'
R="${E}[0m"
B="${E}[1m"
dim="${E}[38;2;110;112;136m"
sub="${E}[38;2;167;166;189m"
txt="${E}[38;2;235;233;243m"
acc="${E}[38;2;240;140;181m"
acc2="${E}[38;2;147;180;242m"
ulc="${E}[58;2;95;97;125m"

rep() { local s="" i; for ((i = 0; i < $2; i++)); do s+="$1"; done; printf '%s' "$s"; }
title() { printf '\n%s%s%s\n\n' "$B" "$1" "$R"; }

title "1. 竖线：连续 4 行，看上下能不能接成一条"
for _ in 1 2 3 4; do
  printf '  %s│%s           %s┃%s           %s▏%s           %s▕%s\n' "$dim" "$R" "$sub" "$R" "$dim" "$R" "$dim" "$R"
done
printf '  %s细线 │      粗线 ┃      贴左 ▏      贴右 ▕%s\n' "$sub" "$R"

title "2. 横线：40 格"
printf '  %s%s%s  %s细线 ─%s\n' "$dim" "$(rep ─ 40)" "$R" "$sub" "$R"
printf '  %s%s%s  %s粗线 ━%s\n' "$sub" "$(rep ━ 40)" "$R" "$sub" "$R"
printf '  %s%s%s  %s贴底 ▁（进度条用）%s\n' "$dim" "$(rep ▁ 40)" "$R" "$sub" "$R"
printf '  %s%s%s  %s贴顶 ▔%s\n' "$dim" "$(rep ▔ 40)" "$R" "$sub" "$R"
printf '  %s%s%s●%s%s%s  %s进度条 ━●─%s\n' "$acc" "$(rep ━ 16)" "$txt" "$dim" "$(rep ─ 23)" "$R" "$sub" "$R"

title "3. 下划线与上划线：不占行，看终端支不支持"
seg="夜を泳ぐ街 · ミナト           "
printf '  %s普通下划线%s  %s%s%s\n' "$sub" "$R" "${E}[4m" "$seg" "$R"
printf '  %s彩色下划线%s  %s%s%s\n' "$sub" "$R" "${E}[4m${ulc}" "$seg" "$R"
printf '  %s点线      %s  %s%s%s\n' "$sub" "$R" "${E}[4:4m${ulc}" "$seg" "$R"
printf '  %s虚线      %s  %s%s%s\n' "$sub" "$R" "${E}[4:5m${ulc}" "$seg" "$R"
printf '  %s波浪线    %s  %s%s%s\n' "$sub" "$R" "${E}[4:3m${ulc}" "$seg" "$R"
printf '  %s上划线    %s  %s%s%s\n' "$sub" "$R" "${E}[53m" "$seg" "$R"

title "4. 拼起来：侧栏 │ 内容，行间下划线，底部 ▁ 进度条"
ul="${E}[4m${ulc}"
printf '  %s▌%s%s♡ 我喜欢的音乐%s  %s│%s %s05  Telephone Line%s          %sPlastic Summer%s   %s5:02%s\n' \
  "$acc" "$R" "$acc$B" "$R" "$dim" "$R" "$txt$B" "$R" "$sub" "$R" "$sub" "$R"
printf '   %s☆ 我的收藏%s      %s│%s %s    %sSQ 原%s%s  %sKaito Mori%s%s%s%s\n' \
  "$sub" "$R" "$dim" "$R" "$ul" "$acc2" "$R" "$ul" "$dim" "$R" "$ul" "$(rep ' ' 29)" "$R"
printf '   %s%s%s  %s│%s %s06  便利店的灯%s              %s便利店的灯%s       %s3:33%s\n' \
  "$dim" "$(rep ─ 14)" "$R" "$dim" "$R" "$txt" "$R" "$sub" "$R" "$sub" "$R"
printf '   %s◷ 最近播放%s      %s│%s %s    %sSQ%s%s  %s七号公路%s%s%s%s\n' \
  "$sub" "$R" "$dim" "$R" "$ul" "$acc2" "$R" "$ul" "$dim" "$R" "$ul" "$(rep ' ' 34)" "$R"
printf '  %s%s%s%s%s\n' "$acc" "$(rep ▁ 26)" "$dim" "$(rep ▁ 42)" "$R"
printf '  %s夜を泳ぐ街%s %s♥%s              %s|◁   ‖   ▷|%s              %s01:12 / 03:47%s\n' \
  "$txt$B" "$R" "$acc" "$R" "$txt" "$R" "$sub" "$R"

if [ -t 0 ] && [ -t 1 ] && [ -z "$NOQUERY" ]; then
  title "5. 宽度检查：这些都是\"模糊宽度\"字符，中文环境下有的终端会当成 2 格"
  old=$(stty -g)
  for ch in '│' '─' '━' '▁' '▌' '♥' '♡' '…' '●'; do
    printf '\r%s' "$ch"
    stty raw -echo min 0 time 10
    printf '\033[6n'
    resp=""
    while IFS= read -r -n1 c; do resp+="$c"; [ "$c" = R ] && break; done
    stty "$old"
    col=${resp##*;}
    col=${col%R}
    if [ -n "$col" ]; then w=$((col - 1)); else w="?"; fi
    if [ "$w" = 1 ]; then note="正常"; elif [ "$w" = "?" ]; then note="终端没回应"; else note="会错位"; fi
    printf '\r  %s  占 %s 格  %s\n' "$ch" "$w" "$note"
  done
fi
printf '\n'
