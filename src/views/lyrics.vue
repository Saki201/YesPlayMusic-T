<template>
  <transition name="slide-up">
    <div
      class="lyrics-page"
      :class="{ 'no-lyric': noLyric && rightPanel !== 'comments' }"
      :data-theme="theme"
    >
      <div
        v-if="
          (settings.lyricsBackground === 'blur') |
            (settings.lyricsBackground === 'dynamic')
        "
        class="lyrics-background"
        :class="{
          'dynamic-background': settings.lyricsBackground === 'dynamic',
        }"
      >
        <div
          class="top-right"
          :style="{ backgroundImage: `url(${bgImageUrl})` }"
        />
        <div
          class="bottom-left"
          :style="{ backgroundImage: `url(${bgImageUrl})` }"
        />
      </div>
      <div
        v-if="settings.lyricsBackground === true"
        class="gradient-background"
        :style="{ background }"
      ></div>

      <div class="left-side">
        <div>
          <div v-if="settings.showLyricsTime" class="date">
            {{ date }}
          </div>
          <div class="cover">
            <div class="cover-container">
              <img :src="imageUrl" loading="lazy" />
              <div
                class="shadow"
                :style="{ backgroundImage: `url(${imageUrl})` }"
              ></div>
            </div>
          </div>
          <div class="controls">
            <div class="top-part">
              <div class="track-info">
                <div class="title" :title="currentTrack.name">
                  <router-link
                    v-if="hasList()"
                    :to="`${getListPath()}`"
                    @click.native="toggleLyrics"
                    >{{ currentTrack.name }}
                  </router-link>
                  <span v-else>
                    {{ currentTrack.name }}
                  </span>
                </div>
                <div class="subtitle">
                  <router-link
                    v-if="artist.id !== 0"
                    :to="`/artist/${artist.id}`"
                    @click.native="toggleLyrics"
                    >{{ artist.name }}
                  </router-link>
                  <span v-else>
                    {{ artist.name }}
                  </span>
                  <span v-if="album.id !== 0">
                    -
                    <router-link
                      :to="`/album/${album.id}`"
                      :title="album.name"
                      @click.native="toggleLyrics"
                      >{{ album.name }}
                    </router-link>
                  </span>
                </div>
              </div>
              <div class="top-right">
                <div class="volume-control">
                  <button-icon :title="$t('player.mute')" @click.native="mute">
                    <svg-icon v-show="volume > 0.5" icon-class="volume" />
                    <svg-icon v-show="volume === 0" icon-class="volume-mute" />
                    <svg-icon
                      v-show="volume <= 0.5 && volume !== 0"
                      icon-class="volume-half"
                    />
                  </button-icon>
                  <div class="volume-bar">
                    <vue-slider
                      v-model="volume"
                      :min="0"
                      :max="1"
                      :interval="0.01"
                      :drag-on-click="true"
                      :duration="0"
                      tooltip="none"
                      :dot-size="12"
                    ></vue-slider>
                  </div>
                </div>
                <div class="buttons">
                  <button-icon
                    :title="$t('player.like')"
                    @click.native="likeATrack(player.currentTrack.id)"
                  >
                    <svg-icon
                      :icon-class="
                        player.isCurrentTrackLiked ? 'heart-solid' : 'heart'
                      "
                    />
                  </button-icon>
                  <button-icon
                    :title="$t('contextMenu.addToPlaylist')"
                    @click.native="addToPlaylist"
                  >
                    <svg-icon icon-class="plus" />
                  </button-icon>
                  <!-- <button-icon @click.native="openMenu" title="Menu"
                    ><svg-icon icon-class="more"
                  /></button-icon> -->
                </div>
              </div>
            </div>
            <div class="progress-bar">
              <span>{{ formatTrackTime(player.progress) || '0:00' }}</span>
              <div class="slider">
                <vue-slider
                  v-model="player.progress"
                  :min="0"
                  :max="player.currentTrackDuration"
                  :interval="1"
                  :drag-on-click="true"
                  :duration="0"
                  :dot-size="12"
                  :height="2"
                  :tooltip-formatter="formatTrackTime"
                  :lazy="true"
                  :silent="true"
                ></vue-slider>
              </div>
              <span>{{ formatTrackTime(player.currentTrackDuration) }}</span>
            </div>
            <div class="media-controls">
              <button-icon
                v-show="!player.isPersonalFM"
                :title="
                  player.repeatMode === 'one'
                    ? $t('player.repeatTrack')
                    : $t('player.repeat')
                "
                :class="{ active: player.repeatMode !== 'off' }"
                @click.native="switchRepeatMode"
              >
                <svg-icon
                  v-show="player.repeatMode !== 'one'"
                  icon-class="repeat"
                />
                <svg-icon
                  v-show="player.repeatMode === 'one'"
                  icon-class="repeat-1"
                />
              </button-icon>
              <div class="middle">
                <button-icon
                  v-show="!player.isPersonalFM"
                  :title="$t('player.previous')"
                  @click.native="playPrevTrack"
                >
                  <svg-icon icon-class="previous" />
                </button-icon>
                <button-icon
                  v-show="player.isPersonalFM"
                  title="不喜欢"
                  @click.native="moveToFMTrash"
                >
                  <svg-icon icon-class="thumbs-down" />
                </button-icon>
                <button-icon
                  id="play"
                  :title="$t(player.playing ? 'player.pause' : 'player.play')"
                  @click.native="playOrPause"
                >
                  <svg-icon :icon-class="player.playing ? 'pause' : 'play'" />
                </button-icon>
                <button-icon
                  :title="$t('player.next')"
                  @click.native="playNextTrack"
                >
                  <svg-icon icon-class="next" />
                </button-icon>
              </div>
              <button-icon
                v-show="!player.isPersonalFM"
                :title="$t('player.shuffle')"
                :class="{ active: player.shuffle }"
                @click.native="switchShuffle"
              >
                <svg-icon icon-class="shuffle" />
              </button-icon>
              <button-icon title="音质选择" @click.native="openQualityMenu">
                <span class="lyric-switch-icon quality-badge">{{
                  qualityLabel
                }}</span>
              </button-icon>
              <button-icon
                :title="rightPanel === 'comments' ? '切换到歌词' : '切换到评论'"
                :class="{ active: rightPanel === 'comments' }"
                @click.native="toggleComments"
              >
                <span class="lyric-switch-icon">{{
                  rightPanel === 'comments' ? '词' : '评'
                }}</span>
              </button-icon>
              <button-icon
                v-show="
                  isShowLyricTypeSwitch &&
                  $store.state.settings.showLyricsTranslation &&
                  lyricType === 'translation'
                "
                :title="$t('player.translationLyric')"
                @click.native="switchLyricType"
              >
                <span class="lyric-switch-icon">译</span>
              </button-icon>
              <button-icon
                v-show="
                  isShowLyricTypeSwitch &&
                  $store.state.settings.showLyricsTranslation &&
                  lyricType === 'romaPronunciation'
                "
                :title="$t('player.PronunciationLyric')"
                @click.native="switchLyricType"
              >
                <span class="lyric-switch-icon">音</span>
              </button-icon>
            </div>
            <ContextMenu ref="qualityMenu">
              <div
                v-for="q in qualityOptions"
                :key="q.value"
                class="item"
                @click="setQuality(q.value)"
              >
                {{ q.label }}
                <span
                  v-if="String(currentQuality) === q.value"
                  style="margin-left: auto"
                  >✓</span
                >
              </div>
            </ContextMenu>
          </div>
        </div>
      </div>
      <div class="right-side">
        <transition name="slide-fade">
          <div
            v-show="!noLyric && rightPanel === 'lyrics'"
            ref="lyricsContainer"
            class="lyrics-container"
            :style="lyricFontSize"
          >
            <div id="line-1" class="line"></div>
            <div
              v-for="(line, index) in lyricToShow"
              :id="`line${index}`"
              :key="index"
              class="line"
              :class="{
                highlight: highlightLyricIndex === index,
              }"
              @click="clickLyricLine(line.time)"
              @dblclick="clickLyricLine(line.time, true)"
            >
              <div class="content">
                <span
                  v-if="line.contents[0]"
                  @click.right="openLyricMenu($event, line, 0)"
                  >{{ line.contents[0] }}</span
                >
                <br />
                <span
                  v-if="
                    line.contents[1] &&
                    $store.state.settings.showLyricsTranslation
                  "
                  class="translation"
                  @click.right="openLyricMenu($event, line, 1)"
                  >{{ line.contents[1] }}</span
                >
              </div>
            </div>
            <ContextMenu v-if="!noLyric" ref="lyricMenu">
              <div class="item" @click="copyLyric(false)">{{
                $t('contextMenu.copyLyric')
              }}</div>
              <div
                v-if="
                  rightClickLyric &&
                  rightClickLyric.contents[1] &&
                  $store.state.settings.showLyricsTranslation
                "
                class="item"
                @click="copyLyric(true)"
                >{{ $t('contextMenu.copyLyricWithTranslation') }}</div
              >
            </ContextMenu>
          </div>
        </transition>
        <transition name="slide-fade">
          <div v-show="rightPanel === 'comments'" class="comments-container">
            <div class="comments-head">
              评论<span v-if="commentTotal">
                ({{ commentTotal | formatPlayCount }})</span
              >
              <div class="sort-tabs">
                <a
                  v-for="s in sortOptions"
                  :key="s.value"
                  class="sort-tab"
                  :class="{ active: commentSortType === s.value }"
                  @click="changeSort(s.value)"
                  >{{ s.label }}</a
                >
              </div>
            </div>
            <div class="comments-list">
              <CommentItem
                v-for="c in comments"
                :key="c.commentId"
                :comment="c"
                @like="likeAComment"
                @reply="startReply"
              >
                <div
                  v-if="c.replyCount"
                  class="expand-replies"
                  @click="toggleReplies(c)"
                  >{{
                    floorOf(c).open ? '收起回复' : `共 ${c.replyCount} 条回复 >`
                  }}</div
                >
                <div v-if="floorOf(c).open" class="replies">
                  <CommentItem
                    v-for="r in floorOf(c).list"
                    :key="r.commentId"
                    :comment="r"
                    is-reply
                    @like="likeAComment"
                    @reply="startReply"
                  />
                  <a
                    v-if="floorOf(c).hasMore && !floorOf(c).loading"
                    class="more-replies"
                    @click="loadFloor(c)"
                    >更多回复</a
                  >
                  <span v-if="floorOf(c).loading" class="more-replies"
                    >加载中...</span
                  >
                </div>
              </CommentItem>
              <button
                v-if="hasMoreComments && !commentsLoading"
                class="load-more"
                @click="loadComments"
                >加载更多</button
              >
              <div v-if="commentsLoading" class="list-hint">加载中...</div>
              <div v-if="!commentsLoading && !comments.length" class="list-hint"
                >暂无评论</div
              >
            </div>
            <div class="comment-input-bar">
              <div v-if="replyTo" class="replying-tip">
                回复 @{{ replyTo.user.nickname }}
                <a @click="cancelReply">取消</a>
              </div>
              <div class="row">
                <input
                  v-model="commentInput"
                  type="text"
                  maxlength="140"
                  :placeholder="
                    replyTo
                      ? `回复 @${replyTo.user.nickname}：`
                      : '随乐而起，有感而发'
                  "
                  @keyup.enter="submitComment"
                  @keydown.stop
                />
                <button
                  :disabled="!commentInput.trim() || commentPosting"
                  @click="submitComment"
                  >发送</button
                >
              </div>
            </div>
          </div>
        </transition>
      </div>
      <div class="close-button" @click="toggleLyrics">
        <button>
          <svg-icon icon-class="arrow-down" />
        </button>
      </div>
      <div class="close-button" style="left: 24px" @click="fullscreen">
        <button>
          <svg-icon v-if="isFullscreen" icon-class="fullscreen-exit" />
          <svg-icon v-else icon-class="fullscreen" />
        </button>
      </div>
    </div>
  </transition>
</template>

<script>
// The lyrics page of Apple Music is so gorgeous, so I copy the design.
// Some of the codes are from https://github.com/sl1673495/vue-netease-music

import { mapState, mapMutations, mapActions } from 'vuex';
import VueSlider from 'vue-slider-component';
import ContextMenu from '@/components/ContextMenu.vue';
import { formatTrackTime } from '@/utils/common';
import { getLyric } from '@/api/track';
import {
  getMusicCommentsNew,
  getCommentFloor,
  likeComment,
  postComment,
} from '@/api/comment';
import { lyricParser, copyLyric } from '@/utils/lyrics';
import ButtonIcon from '@/components/ButtonIcon.vue';
import CommentItem from '@/components/CommentItem.vue';
import * as Vibrant from 'node-vibrant/dist/vibrant.worker.min.js';
import Color from 'color';
import { isAccountLoggedIn } from '@/utils/auth';
import { hasListSource, getListSourcePath } from '@/utils/playList';
import locale from '@/locale';

const electron =
  process.env.IS_ELECTRON === true ? window.require('electron') : null;
const ipcRenderer =
  process.env.IS_ELECTRON === true ? electron.ipcRenderer : null;

export default {
  name: 'Lyrics',
  components: {
    VueSlider,
    ButtonIcon,
    ContextMenu,
    CommentItem,
  },
  data() {
    return {
      lyricsInterval: null,
      lyric: [],
      tlyric: [],
      romalyric: [],
      lyricType: 'translation', // or 'romaPronunciation'
      highlightLyricIndex: -1,
      minimize: true,
      background: '',
      date: this.formatTime(new Date()),
      isFullscreen: process.env.IS_TAURI
        ? false
        : !!document.fullscreenElement,
      rightClickLyric: null,
      // 右侧面板：歌词 / 评论
      rightPanel: 'lyrics',
      comments: [],
      commentTotal: 0,
      commentPageNo: 1,
      commentCursor: '',
      commentSortType: 2, // 1 推荐 / 2 热度 / 3 时间
      hasMoreComments: false,
      commentsLoading: false,
      floorState: {}, // commentId -> { open, list, hasMore, time, loading }
      replyTo: null,
      commentInput: '',
      commentPosting: false,
    };
  },
  computed: {
    ...mapState(['player', 'settings', 'showLyrics']),
    currentTrack() {
      return this.player.currentTrack;
    },
    volume: {
      get() {
        return this.player.volume;
      },
      set(value) {
        this.player.volume = value;
      },
    },
    imageUrl() {
      return this.player.currentTrack?.al?.picUrl + '?param=1024y1024';
    },
    bgImageUrl() {
      return this.player.currentTrack?.al?.picUrl + '?param=512y512';
    },
    isShowLyricTypeSwitch() {
      return this.romalyric.length > 0 && this.tlyric.length > 0;
    },
    lyricToShow() {
      return this.lyricType === 'translation'
        ? this.lyricWithTranslation
        : this.lyricWithRomaPronunciation;
    },
    lyricWithTranslation() {
      let ret = [];
      // 空内容的去除
      const lyricFiltered = this.lyric.filter(({ content }) =>
        Boolean(content)
      );
      // content统一转换数组形式
      if (lyricFiltered.length) {
        lyricFiltered.forEach(l => {
          const { rawTime, time, content } = l;
          const lyricItem = { time, content, contents: [content] };
          const sameTimeTLyric = this.tlyric.find(
            ({ rawTime: tLyricRawTime }) => tLyricRawTime === rawTime
          );
          if (sameTimeTLyric) {
            const { content: tLyricContent } = sameTimeTLyric;
            if (content) {
              lyricItem.contents.push(tLyricContent);
            }
          }
          ret.push(lyricItem);
        });
      } else {
        ret = lyricFiltered.map(({ time, content }) => ({
          time,
          content,
          contents: [content],
        }));
      }
      return ret;
    },
    lyricWithRomaPronunciation() {
      let ret = [];
      // 空内容的去除
      const lyricFiltered = this.lyric.filter(({ content }) =>
        Boolean(content)
      );
      // content统一转换数组形式
      if (lyricFiltered.length) {
        lyricFiltered.forEach(l => {
          const { rawTime, time, content } = l;
          const lyricItem = { time, content, contents: [content] };
          const sameTimeRomaLyric = this.romalyric.find(
            ({ rawTime: tLyricRawTime }) => tLyricRawTime === rawTime
          );
          if (sameTimeRomaLyric) {
            const { content: romaLyricContent } = sameTimeRomaLyric;
            if (content) {
              lyricItem.contents.push(romaLyricContent);
            }
          }
          ret.push(lyricItem);
        });
      } else {
        ret = lyricFiltered.map(({ time, content }) => ({
          time,
          content,
          contents: [content],
        }));
      }
      return ret;
    },
    lyricFontSize() {
      return {
        fontSize: `${this.$store.state.settings.lyricFontSize || 28}px`,
      };
    },
    noLyric() {
      return this.lyric.length == 0;
    },
    artist() {
      return this.currentTrack?.ar
        ? this.currentTrack.ar[0]
        : { id: 0, name: 'unknown' };
    },
    album() {
      return this.currentTrack?.al || { id: 0, name: 'unknown' };
    },
    theme() {
      return this.settings.lyricsBackground === true ? 'dark' : 'auto';
    },
    currentQuality() {
      return this.settings.musicQuality ?? 320000;
    },
    qualityOptions() {
      return [
        { value: '128000', label: '标准 128K' },
        { value: '192000', label: '较高 192K' },
        { value: '320000', label: '极高 320K' },
        { value: 'flac', label: '无损 FLAC' },
        { value: '999000', label: 'Hi-Res' },
      ];
    },
    qualityLabel() {
      const map = {
        128000: '128K',
        192000: '192K',
        320000: '320K',
        flac: '无损',
        999000: 'Hi-Res',
      };
      return map[String(this.currentQuality)] ?? '320K';
    },
    sortOptions() {
      return [
        { value: 1, label: '推荐' },
        { value: 2, label: '最热' },
        { value: 3, label: '最新' },
      ];
    },
  },
  watch: {
    currentTrack(newVal, oldVal) {
      this.getLyric();
      this.getCoverColor();
      // 切歌时重置评论（乐观更新会让 currentTrack 对象替换两次，按 id 去重）
      if (newVal?.id !== oldVal?.id) {
        this.resetComments();
        if (this.rightPanel === 'comments') {
          this.loadComments();
        }
      }
    },
    showLyrics(show) {
      if (show) {
        this.setLyricsInterval();
        this.$store.commit('enableScrolling', false);
      } else {
        clearInterval(this.lyricsInterval);
        this.$store.commit('enableScrolling', true);
      }
    },
  },
  created() {
    this.getLyric();
    this.getCoverColor();
    this.initDate();
    document.addEventListener('keydown', e => {
      if (e.key === 'F11') {
        e.preventDefault();
        this.fullscreen();
      }
    });
    // Tauri：原生窗口全屏，监听 Rust 端 fullscreenChanged 事件
    // 非 Tauri（Electron / 浏览器）：监听浏览器 fullscreenchange 事件
    if (process.env.IS_TAURI) {
      ipcRenderer.on('fullscreenChanged', (_, value) => {
        this.isFullscreen = value;
      });
    } else {
      document.addEventListener('fullscreenchange', () => {
        this.isFullscreen = !!document.fullscreenElement;
      });
    }
  },
  beforeDestroy: function () {
    if (this.timer) {
      clearInterval(this.timer);
    }
  },
  destroyed() {
    clearInterval(this.lyricsInterval);
  },
  methods: {
    ...mapMutations(['toggleLyrics', 'updateModal']),
    ...mapActions(['likeATrack']),
    initDate() {
      var _this = this;
      clearInterval(this.timer);
      this.timer = setInterval(function () {
        _this.date = _this.formatTime(new Date());
      }, 1000);
    },
    formatTime(value) {
      let hour = value.getHours().toString();
      let minute = value.getMinutes().toString();
      let second = value.getSeconds().toString();
      return (
        hour.padStart(2, '0') +
        ':' +
        minute.padStart(2, '0') +
        ':' +
        second.padStart(2, '0')
      );
    },
    fullscreen() {
      // Tauri：使用原生窗口全屏（set_fullscreen），避免 WebView2 最大化→全屏黑条问题
      if (process.env.IS_TAURI) {
        ipcRenderer.send('fullscreen');
      } else if (document.fullscreenElement) {
        document.exitFullscreen();
      } else {
        document.documentElement.requestFullscreen();
      }
    },
    addToPlaylist() {
      if (!isAccountLoggedIn()) {
        this.showToast(locale.t('toast.needToLogin'));
        return;
      }
      this.$store.dispatch('fetchLikedPlaylist');
      this.updateModal({
        modalName: 'addTrackToPlaylistModal',
        key: 'show',
        value: true,
      });
      this.updateModal({
        modalName: 'addTrackToPlaylistModal',
        key: 'selectedTrackID',
        value: this.currentTrack?.id,
      });
    },
    playPrevTrack() {
      this.player.playPrevTrack();
    },
    playOrPause() {
      this.player.playOrPause();
    },
    playNextTrack() {
      if (this.player.isPersonalFM) {
        this.player.playNextFMTrack();
      } else {
        this.player.playNextTrack();
      }
    },
    openQualityMenu(e) {
      this.$refs.qualityMenu.openMenu(e);
    },
    setQuality(value) {
      if (String(this.currentQuality) === value) return;
      this.$store.commit('changeMusicQuality', value);
      this.player.switchQuality();
      const opt = this.qualityOptions.find(q => q.value === value);
      this.$store.dispatch('showToast', `已切换至${opt?.label ?? value}`);
    },
    toggleComments() {
      this.rightPanel = this.rightPanel === 'comments' ? 'lyrics' : 'comments';
      if (
        this.rightPanel === 'comments' &&
        !this.comments.length &&
        !this.commentsLoading
      ) {
        this.loadComments();
      }
    },
    resetComments() {
      this.comments = [];
      this.commentTotal = 0;
      this.commentPageNo = 1;
      this.commentCursor = '';
      this.hasMoreComments = false;
      this.floorState = {};
      this.replyTo = null;
    },
    changeSort(sortType) {
      this.commentSortType = sortType;
      this.resetComments();
      this.loadComments();
    },
    loadComments() {
      const id = this.currentTrack?.id;
      if (!id || this.commentsLoading) return;
      this.commentsLoading = true;
      const pageNo = this.commentPageNo;
      const params = {
        id,
        pageNo,
        pageSize: 30,
        sortType: this.commentSortType,
      };
      // sortType=3（时间排序）翻页需要 cursor
      if (this.commentSortType === 3 && pageNo > 1) {
        params.cursor = this.commentCursor;
      }
      getMusicCommentsNew(params)
        .then(res => {
          const data = res?.data;
          // 请求期间已切歌则丢弃过期数据
          if (!data || this.currentTrack?.id !== id) return;
          this.comments =
            pageNo === 1
              ? data.comments ?? []
              : this.comments.concat(data.comments ?? []);
          this.commentTotal = data.totalCount ?? 0;
          this.hasMoreComments = data.hasMore ?? false;
          this.commentPageNo = pageNo + 1;
          const last = this.comments[this.comments.length - 1];
          this.commentCursor = last?.time ?? '';
        })
        .finally(() => {
          this.commentsLoading = false;
        });
    },
    likeAComment(comment) {
      if (!isAccountLoggedIn()) {
        this.$store.dispatch('showToast', locale.t('toast.needToLogin'));
        return;
      }
      const t = comment.liked ? 0 : 1;
      // 乐观更新，失败回滚
      comment.liked = !comment.liked;
      comment.likedCount = (comment.likedCount || 0) + (t === 1 ? 1 : -1);
      likeComment(this.currentTrack.id, comment.commentId, t)
        .then(res => {
          if (res?.code !== 200) throw new Error(res?.msg);
        })
        .catch(() => {
          comment.liked = !comment.liked;
          comment.likedCount = (comment.likedCount || 0) + (t === 1 ? -1 : 1);
          this.$store.dispatch('showToast', '操作失败');
        });
    },
    startReply(comment) {
      if (!isAccountLoggedIn()) {
        this.$store.dispatch('showToast', locale.t('toast.needToLogin'));
        return;
      }
      this.replyTo = comment;
    },
    cancelReply() {
      this.replyTo = null;
    },
    submitComment() {
      if (!isAccountLoggedIn()) {
        this.$store.dispatch('showToast', locale.t('toast.needToLogin'));
        return;
      }
      const content = this.commentInput.trim();
      if (!content || this.commentPosting) return;
      this.commentPosting = true;
      const replyTo = this.replyTo;
      const params = replyTo
        ? {
            id: this.currentTrack.id,
            content,
            t: 2,
            commentId: replyTo.commentId,
          }
        : { id: this.currentTrack.id, content, t: 1 };
      postComment(params)
        .then(res => {
          if (res?.code !== 200) {
            this.$store.dispatch(
              'showToast',
              res?.msg ?? res?.message ?? '评论失败'
            );
            return;
          }
          this.$store.dispatch('showToast', replyTo ? '回复成功' : '评论成功');
          this.commentInput = '';
          this.replyTo = null;
          // 刷新列表（新评论可能有审核延迟，未必立即可见）
          this.changeSort(this.commentSortType);
        })
        .finally(() => {
          this.commentPosting = false;
        });
    },
    floorOf(comment) {
      return (
        this.floorState[comment.commentId] ?? {
          open: false,
          list: [],
          hasMore: false,
          loading: false,
        }
      );
    },
    toggleReplies(comment) {
      const state = this.floorState[comment.commentId];
      if (!state) {
        this.$set(this.floorState, comment.commentId, {
          open: true,
          list: [],
          hasMore: false,
          time: undefined,
          loading: false,
        });
        this.loadFloor(comment);
      } else {
        state.open = !state.open;
      }
    },
    loadFloor(comment) {
      const state = this.floorState[comment.commentId];
      if (!state || state.loading) return;
      state.loading = true;
      getCommentFloor({
        parentCommentId: comment.commentId,
        id: this.currentTrack.id,
        limit: 20,
        time: state.time,
      })
        .then(res => {
          const data = res?.data;
          if (!data) return;
          state.list = state.list.concat(data.comments ?? []);
          state.hasMore = data.hasMore ?? false;
          state.time = data.time ?? state.time;
        })
        .finally(() => {
          state.loading = false;
        });
    },
    getLyric() {
      if (!this.currentTrack.id) return;
      return getLyric(this.currentTrack.id).then(data => {
        if (!data?.lrc?.lyric) {
          this.lyric = [];
          this.tlyric = [];
          this.romalyric = [];
          return false;
        } else {
          let { lyric, tlyric, romalyric } = lyricParser(data);
          lyric = lyric.filter(
            l => !/^作(词|曲)\s*(:|：)\s*无$/.exec(l.content)
          );
          let includeAM =
            lyric.length <= 10 &&
            lyric.map(l => l.content).includes('纯音乐，请欣赏');
          if (includeAM) {
            let reg = /^作(词|曲)\s*(:|：)\s*/;
            let author = this.currentTrack?.ar[0]?.name;
            lyric = lyric.filter(l => {
              let regExpArr = l.content.match(reg);
              return (
                !regExpArr || l.content.replace(regExpArr[0], '') !== author
              );
            });
          }
          if (lyric.length === 1 && includeAM) {
            this.lyric = [];
            this.tlyric = [];
            this.romalyric = [];
            return false;
          } else {
            this.lyric = lyric;
            this.tlyric = tlyric;
            this.romalyric = romalyric;
            if (tlyric.length * romalyric.length > 0) {
              this.lyricType = 'translation';
            } else {
              this.lyricType =
                lyric.length > 0 ? 'translation' : 'romaPronunciation';
            }
            return true;
          }
        }
      });
    },
    switchLyricType() {
      this.lyricType =
        this.lyricType === 'translation' ? 'romaPronunciation' : 'translation';
    },
    formatTrackTime(value) {
      return formatTrackTime(value);
    },
    clickLyricLine(value, startPlay = false) {
      // TODO: 双击选择还会选中文字，考虑搞个右键菜单复制歌词
      let jumpFlag = false;
      this.lyric.filter(function (item) {
        if (item.content == '纯音乐，请欣赏') {
          jumpFlag = true;
        }
      });
      if (window.getSelection().toString().length === 0 && !jumpFlag) {
        this.player.seek(value);
      }
      if (startPlay === true) {
        this.player.play();
      }
    },
    openLyricMenu(e, lyric, idx) {
      this.rightClickLyric = { ...lyric, idx };
      this.$refs.lyricMenu.openMenu(e);
      e.preventDefault();
    },
    copyLyric(withTranslation) {
      if (this.rightClickLyric) {
        const idx = this.rightClickLyric.idx;
        if (!withTranslation) {
          copyLyric(this.rightClickLyric.contents[idx]);
        } else {
          copyLyric(this.rightClickLyric.contents.join(' '));
        }
      }
    },
    setLyricsInterval() {
      this.lyricsInterval = setInterval(() => {
        const progress = this.player.seek(null, false) ?? 0;
        let oldHighlightLyricIndex = this.highlightLyricIndex;
        this.highlightLyricIndex = this.lyric.findIndex((l, index) => {
          const nextLyric = this.lyric[index + 1];
          return (
            progress >= l.time && (nextLyric ? progress < nextLyric.time : true)
          );
        });
        if (oldHighlightLyricIndex !== this.highlightLyricIndex) {
          const el = document.getElementById(`line${this.highlightLyricIndex}`);
          if (el)
            el.scrollIntoView({
              behavior: 'smooth',
              block: 'center',
            });
        }
      }, 50);
    },
    moveToFMTrash() {
      this.player.moveToFMTrash();
    },
    switchRepeatMode() {
      this.player.switchRepeatMode();
    },
    switchShuffle() {
      this.player.switchShuffle();
    },
    getCoverColor() {
      if (this.settings.lyricsBackground !== true) return;
      const cover = this.currentTrack.al?.picUrl + '?param=256y256';
      Vibrant.from(cover, { colorCount: 1 })
        .getPalette()
        .then(palette => {
          const originColor = Color.rgb(palette.DarkMuted._rgb);
          const color = originColor.darken(0.1).rgb().string();
          const color2 = originColor.lighten(0.28).rotate(-30).rgb().string();
          this.background = `linear-gradient(to top left, ${color}, ${color2})`;
        });
    },
    hasList() {
      return hasListSource();
    },
    getListPath() {
      return getListSourcePath();
    },
    mute() {
      this.player.mute();
    },
  },
};
</script>

<style lang="scss" scoped>
.lyrics-page {
  position: fixed;
  top: 0;
  right: 0;
  left: 0;
  bottom: 0;
  z-index: 200;
  background: var(--color-body-bg);
  display: flex;
  clip: rect(auto, auto, auto, auto);
}

.lyrics-background {
  --contrast-lyrics-background: 75%;
  --brightness-lyrics-background: 150%;
}

[data-theme='dark'] .lyrics-background {
  --contrast-lyrics-background: 125%;
  --brightness-lyrics-background: 50%;
}

.lyrics-background {
  filter: blur(50px) contrast(var(--contrast-lyrics-background))
    brightness(var(--brightness-lyrics-background));
  position: absolute;
  height: 100vh;
  width: 100vw;
  .top-right,
  .bottom-left {
    z-index: 0;
    width: 140vw;
    height: 140vw;
    opacity: 0.6;
    position: absolute;
    background-size: cover;
  }

  .top-right {
    right: 0;
    top: 0;
    mix-blend-mode: luminosity;
  }

  .bottom-left {
    left: 0;
    bottom: 0;
    animation-direction: reverse;
    animation-delay: 10s;
  }
}

.dynamic-background > div {
  animation: rotate 150s linear infinite;
}

@keyframes rotate {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}

.gradient-background {
  position: absolute;
  height: 100vh;
  width: 100vw;
}

.left-side {
  flex: 1;
  display: flex;
  justify-content: flex-end;
  margin-right: 32px;
  margin-top: 24px;
  align-items: center;
  transition: all 0.5s;

  z-index: 1;

  .date {
    max-width: 54vh;
    margin: 24px 0;
    color: var(--color-text);
    text-align: center;
    font-size: 4rem;
    font-weight: 600;
    opacity: 0.88;
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 1;
    overflow: hidden;
  }

  .controls {
    max-width: 54vh;
    margin-top: 24px;
    color: var(--color-text);

    .title {
      margin-top: 8px;
      font-size: 1.4rem;
      font-weight: 600;
      opacity: 0.88;
      display: -webkit-box;
      -webkit-box-orient: vertical;
      -webkit-line-clamp: 1;
      overflow: hidden;
    }

    .subtitle {
      margin-top: 4px;
      font-size: 1rem;
      opacity: 0.58;
      display: -webkit-box;
      -webkit-box-orient: vertical;
      -webkit-line-clamp: 1;
      overflow: hidden;
    }

    .top-part {
      display: flex;
      justify-content: space-between;

      .top-right {
        display: flex;
        justify-content: space-between;

        .volume-control {
          margin: 0 10px;
          display: flex;
          align-items: center;
          .volume-bar {
            width: 84px;
          }
        }

        .buttons {
          display: flex;
          align-items: center;

          button {
            margin: 0 0 0 4px;
          }

          .svg-icon {
            height: 18px;
            width: 18px;
          }
        }
      }
    }

    .progress-bar {
      margin-top: 22px;
      display: flex;
      align-items: center;
      justify-content: space-between;

      .slider {
        width: 100%;
        flex-grow: grow;
        padding: 0 10px;
      }

      span {
        font-size: 15px;
        opacity: 0.58;
        min-width: 28px;
      }
    }

    .media-controls {
      display: flex;
      justify-content: center;
      margin-top: 18px;
      align-items: center;

      button {
        margin: 0;
      }

      .svg-icon {
        opacity: 0.38;
        height: 14px;
        width: 14px;
      }

      .active .svg-icon {
        opacity: 0.88;
      }

      .middle {
        padding: 0 16px;
        display: flex;
        align-items: center;

        button {
          margin: 0 8px;
        }

        button#play .svg-icon {
          height: 28px;
          width: 28px;
          padding: 2px;
        }

        .svg-icon {
          opacity: 0.88;
          height: 22px;
          width: 22px;
        }
      }
      .lyric-switch-icon {
        color: var(--color-text);
        font-size: 14px;
        line-height: 14px;
        opacity: 0.88;
      }
      .quality-badge {
        font-size: 11px;
        line-height: 12px;
        font-weight: 700;
        border: 1.5px solid var(--color-text);
        border-radius: 4px;
        padding: 1px 4px;
        white-space: nowrap;
      }
    }
  }
}

.cover {
  position: relative;

  .cover-container {
    position: relative;
  }

  img {
    border-radius: 0.75em;
    width: 54vh;
    height: 54vh;
    user-select: none;
    object-fit: cover;
  }

  .shadow {
    position: absolute;
    top: 12px;
    height: 54vh;
    width: 54vh;
    filter: blur(16px) opacity(0.6);
    transform: scale(0.92, 0.96);
    z-index: -1;
    background-size: cover;
    border-radius: 0.75em;
  }
}

.right-side {
  flex: 1;
  font-weight: 600;
  color: var(--color-text);
  margin-right: 24px;
  z-index: 0;

  .lyrics-container {
    height: 100%;
    display: flex;
    flex-direction: column;
    padding-left: 78px;
    max-width: 460px;
    overflow-y: auto;
    transition: 0.5s;
    scrollbar-width: none; // firefox

    .line {
      margin: 2px 0;
      padding: 12px 18px;
      transition: 0.5s;
      border-radius: 12px;

      &:hover {
        background: var(--color-secondary-bg-for-transparent);
      }

      .content {
        transform-origin: center left;
        transform: scale(0.95);
        transition: all 0.35s cubic-bezier(0.25, 0.46, 0.45, 0.94);
        user-select: none;

        span {
          opacity: 0.28;
          cursor: default;
          font-size: 1em;
          transition: all 0.35s cubic-bezier(0.25, 0.46, 0.45, 0.94);
        }

        span.translation {
          opacity: 0.2;
          font-size: 0.925em;
        }
      }
    }

    .line#line-1:hover {
      background: unset;
    }

    .translation {
      margin-top: 0.1em;
    }

    .highlight div.content {
      transform: scale(1);
      span {
        opacity: 0.98;
        display: inline-block;
      }

      span.translation {
        opacity: 0.65;
      }
    }
  }

  ::-webkit-scrollbar {
    display: none;
  }

  .lyrics-container .line:first-child {
    margin-top: 50vh;
  }

  .lyrics-container .line:last-child {
    margin-bottom: calc(50vh - 128px);
  }

  .comments-container {
    height: 100vh;
    display: flex;
    flex-direction: column;
    padding-left: 78px;
    max-width: 460px;

    .comments-head {
      font-size: 28px;
      font-weight: 700;
      padding: 6vh 18px 8px 18px;

      span {
        font-size: 16px;
        opacity: 0.58;
      }

      .sort-tabs {
        margin-top: 10px;
        display: flex;
        font-size: 14px;

        .sort-tab {
          margin-right: 16px;
          opacity: 0.58;
          cursor: pointer;

          &:hover {
            opacity: 0.88;
          }

          &.active {
            opacity: 1;
            color: var(--color-primary);
          }
        }
      }
    }

    .comments-list {
      flex: 1;
      overflow-y: auto;
      padding-bottom: 16px;

      .expand-replies {
        font-size: 13px;
        font-weight: 600;
        color: var(--color-primary);
        margin-top: 6px;
        cursor: pointer;
        width: fit-content;
      }

      .replies {
        margin-top: 4px;
        padding-left: 12px;
        border-left: 2px solid var(--color-secondary-bg-for-transparent);
      }

      .more-replies {
        display: inline-block;
        font-size: 13px;
        font-weight: 600;
        opacity: 0.58;
        margin: 8px 0 4px 0;
        cursor: pointer;

        &:hover {
          opacity: 0.88;
        }
      }

      .load-more {
        margin: 16px 18px 0 18px;
        background: var(--color-secondary-bg-for-transparent);
        color: var(--color-text);
        border: none;
        border-radius: 8px;
        padding: 8px 16px;
        font-size: 14px;
        font-weight: 600;
        cursor: pointer;

        &:hover {
          opacity: 0.88;
        }
      }

      .list-hint {
        margin: 16px 18px 0 18px;
        font-size: 14px;
        opacity: 0.48;
      }
    }

    .comment-input-bar {
      padding: 12px 18px 4vh 18px;

      .replying-tip {
        font-size: 13px;
        opacity: 0.68;
        margin-bottom: 6px;

        a {
          color: var(--color-primary);
          cursor: pointer;
          margin-left: 8px;
        }
      }

      .row {
        display: flex;

        input {
          flex: 1;
          border: none;
          background: var(--color-secondary-bg-for-transparent);
          border-radius: 8px;
          padding: 9px 12px;
          color: var(--color-text);
          font-size: 14px;
          font-weight: 500;
          outline: none;

          &::placeholder {
            color: var(--color-text);
            opacity: 0.38;
          }
        }

        button {
          margin-left: 8px;
          border: none;
          border-radius: 8px;
          padding: 0 16px;
          background: var(--color-primary-bg-for-transparent);
          color: var(--color-primary);
          font-size: 14px;
          font-weight: 600;
          cursor: pointer;

          &:disabled {
            opacity: 0.38;
            cursor: default;
          }
        }
      }
    }
  }
}

.close-button {
  position: fixed;
  top: 24px;
  right: 24px;
  z-index: 300;
  border-radius: 0.75rem;
  height: 44px;
  width: 44px;
  display: flex;
  justify-content: center;
  align-items: center;
  opacity: 0.28;
  transition: 0.2s;
  -webkit-app-region: no-drag;

  .svg-icon {
    color: var(--color-text);
    padding-top: 5px;
    height: 22px;
    width: 22px;
  }

  &:hover {
    background: var(--color-secondary-bg-for-transparent);
    opacity: 0.88;
  }
}

.lyrics-page.no-lyric {
  .left-side {
    transition: all 0.5s;
    transform: translateX(27vh);
    margin-right: 0;
  }
}

@media (max-aspect-ratio: 10/9) {
  .left-side {
    display: none;
  }
  .right-side .lyrics-container {
    max-width: 100%;
  }
}

@media screen and (min-width: 1200px) {
  .right-side .lyrics-container {
    max-width: 600px;
  }
}

.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.4s;
}

.slide-up-enter, .slide-up-leave-to /* .fade-leave-active below version 2.1.8 */ {
  transform: translateY(100%);
}

.slide-fade-enter-active {
  transition: all 0.5s ease;
}

.slide-fade-leave-active {
  transition: all 0.5s cubic-bezier(0.2, 0.2, 0, 1);
}

.slide-fade-enter,
.slide-fade-leave-to {
  transform: translateX(27vh);
  opacity: 0;
}
</style>
