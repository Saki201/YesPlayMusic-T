import request from '@/utils/request';

/**
 * 歌曲评论（旧版接口）
 * @param {number} id - 音乐 id
 * @param {number} limit - 取出评论数量, 默认为 20
 * @param {number} offset - 偏移数量, 用于分页
 */
export function getMusicComments(id, limit = 20, offset = 0) {
  return request({
    url: '/comment/music',
    method: 'get',
    params: {
      id,
      limit,
      offset,
      timestamp: new Date().getTime(),
    },
  });
}

/**
 * 歌曲评论（新版接口，带 replyCount/liked 等字段，官方客户端同款）
 * @param {Object} params
 * @param {number} params.id - 音乐 id
 * @param {number} params.pageNo - 页码（从 1 开始）
 * @param {number} params.pageSize - 每页数量
 * @param {number} params.sortType - 排序：1 推荐 / 2 热度 / 3 时间
 * @param {string=} params.cursor - sortType=3 翻页时传上一页最后一条的 time
 */
export function getMusicCommentsNew(params) {
  return request({
    url: '/comment/new',
    method: 'get',
    params: {
      type: 0,
      ...params,
      timestamp: new Date().getTime(),
    },
  });
}

/**
 * 楼层评论（展开某条评论的回复）
 * @param {Object} params
 * @param {number} params.parentCommentId - 父评论 id
 * @param {number} params.id - 音乐 id
 * @param {number=} params.limit
 * @param {number=} params.time - 翻页用，上一页响应中的 time
 */
export function getCommentFloor(params) {
  return request({
    url: '/comment/floor',
    method: 'get',
    params: {
      type: 0,
      ...params,
      timestamp: new Date().getTime(),
    },
  });
}

/**
 * 点赞/取消点赞评论
 * @param {number} id - 音乐 id
 * @param {number} cid - 评论 id
 * @param {number} t - 1 点赞 / 0 取消点赞
 */
export function likeComment(id, cid, t) {
  return request({
    url: '/comment/like',
    method: 'get',
    params: {
      id,
      cid,
      t,
      type: 0,
      timestamp: new Date().getTime(),
    },
  });
}

/**
 * 发表/回复评论
 * @param {Object} params
 * @param {number} params.id - 音乐 id
 * @param {string} params.content - 评论内容
 * @param {number} params.t - 1 发送 / 2 回复
 * @param {number=} params.commentId - 回复时传被回复评论的 id
 */
export function postComment(params) {
  return request({
    url: '/comment',
    method: 'post',
    params: {
      type: 0,
      ...params,
      timestamp: new Date().getTime(),
    },
  });
}
