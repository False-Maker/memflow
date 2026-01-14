// Web Worker 用于计算知识图谱的力导向布局
// 避免阻塞 UI 主线程

self.onmessage = function(e) {
  const { nodes, edges, width, height } = e.data;
  
  // 简单的力导向布局算法
  const iterations = 100;
  const k = Math.sqrt((width * height) / nodes.length);
  const repulsion = k * k;
  const attraction = 0.1;
  
  // 初始化位置
  nodes.forEach((node: any) => {
    node.x = Math.random() * width;
    node.y = Math.random() * height;
    node.vx = 0;
    node.vy = 0;
  });
  
  // 迭代计算
  for (let iter = 0; iter < iterations; iter++) {
    // 计算排斥力
    for (let i = 0; i < nodes.length; i++) {
      let fx = 0;
      let fy = 0;
      
      for (let j = 0; j < nodes.length; j++) {
        if (i === j) continue;
        
        const dx = nodes[i].x - nodes[j].x;
        const dy = nodes[i].y - nodes[j].y;
        const dist = Math.sqrt(dx * dx + dy * dy) || 1;
        
        fx += (repulsion / dist) * (dx / dist);
        fy += (repulsion / dist) * (dy / dist);
      }
      
      nodes[i].vx = (nodes[i].vx + fx) * 0.9;
      nodes[i].vy = (nodes[i].vy + fy) * 0.9;
    }
    
    // 计算吸引力
    edges.forEach((edge: any) => {
      const source = nodes.find((n: any) => n.id === edge.source);
      const target = nodes.find((n: any) => n.id === edge.target);
      
      if (!source || !target) return;
      
      const dx = target.x - source.x;
      const dy = target.y - source.y;
      const dist = Math.sqrt(dx * dx + dy * dy) || 1;
      
      const force = dist * attraction;
      
      source.vx += (dx / dist) * force;
      source.vy += (dy / dist) * force;
      target.vx -= (dx / dist) * force;
      target.vy -= (dy / dist) * force;
    });
    
    // 更新位置
    nodes.forEach((node: any) => {
      node.x += node.vx;
      node.y += node.vy;
      
      // 边界约束
      node.x = Math.max(0, Math.min(width, node.x));
      node.y = Math.max(0, Math.min(height, node.y));
    });
    
    // 每 10 次迭代发送一次进度
    if (iter % 10 === 0) {
      self.postMessage({ type: 'progress', progress: iter / iterations, nodes });
    }
  }
  
  // 发送最终结果
  self.postMessage({ type: 'complete', nodes });
};

