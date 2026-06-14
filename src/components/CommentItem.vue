<template>
  <div class="comment-item" :class="{ reply: isReply }">
    <img
      class="avatar"
      :src="comment.user.avatarUrl | resizeImage(64)"
      loading="lazy"
    />
    <div class="comment-main">
      <div class="nickname">{{ comment.user.nickname }}</div>
      <div class="content">
        <span v-if="replyPrefix" class="reply-prefix"
          >回复 @{{ replyPrefix }}：</span
        >{{ comment.content }}
      </div>
      <div class="meta">
        <span class="time"
          >{{ comment.time | formatDate
          }}<template v-if="location"> · {{ location }}</template></span
        >
        <div class="actions">
          <a
            class="like"
            :class="{ liked: comment.liked }"
            @click="$emit('like', comment)"
            >{{ comment.liked ? '♥' : '♡'
            }}<template v-if="comment.likedCount">
              {{ comment.likedCount | formatPlayCount }}</template
            ></a
          >
          <a @click="$emit('reply', comment)">回复</a>
        </div>
      </div>
      <slot></slot>
    </div>
  </div>
</template>

<script>
export default {
  name: 'CommentItem',
  props: {
    comment: { type: Object, required: true },
    isReply: { type: Boolean, default: false },
  },
  computed: {
    replyPrefix() {
      // 楼层内回复他人时显示 @昵称
      if (!this.isReply) return '';
      const be = this.comment.beReplied?.[0];
      return be?.user?.nickname ?? '';
    },
    location() {
      return this.comment.ipLocation?.location ?? '';
    },
  },
};
</script>

<style lang="scss" scoped>
.comment-item {
  display: flex;
  padding: 10px 18px;
  border-radius: 12px;

  &:hover {
    background: var(--color-secondary-bg-for-transparent);
  }

  .avatar {
    height: 36px;
    width: 36px;
    border-radius: 50%;
    margin-right: 12px;
    flex-shrink: 0;
  }

  .comment-main {
    flex: 1;
    min-width: 0;

    .nickname {
      font-size: 13px;
      opacity: 0.58;
    }

    .content {
      font-size: 15px;
      font-weight: 500;
      margin: 4px 0;
      line-height: 1.5;
      word-break: break-word;
      user-select: text;

      .reply-prefix {
        opacity: 0.58;
      }
    }

    .meta {
      display: flex;
      justify-content: space-between;
      align-items: center;
      font-size: 12px;
      opacity: 0.68;

      .actions {
        display: flex;

        a {
          margin-left: 14px;
          cursor: pointer;

          &:hover {
            opacity: 1;
          }

          &.like.liked {
            color: var(--color-primary);
          }
        }
      }
    }
  }

  &.reply {
    padding: 8px 0 0 0;

    &:hover {
      background: unset;
    }

    .avatar {
      height: 28px;
      width: 28px;
      margin-right: 10px;
    }

    .comment-main .content {
      font-size: 14px;
    }
  }
}
</style>
