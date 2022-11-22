// 例子：定义一个聊天服务的数据结构

// #[derive(Debug)] 为数据结构实现了 Debug trait，提供了 debug 能力，这样可以通过 {:?}，用 println! 打印出来
// Clone派生宏的作用：让数据结构可以复制
// Copy派生宏的作用：让数据结构可以在参数传递时自动按字节拷贝

// 性别枚举
#[derive(Debug)]
enum Gender {
    Female = 1,
    Male = 2,
}

// 用户id 是一个元组结构体
#[derive(Debug, Copy, Clone)]
struct UserId(u64);

// 帖子id
#[derive(Debug, Copy, Clone)]
struct TopicId(u64);

// 用户信息 是一个标准的结构体
#[derive(Debug)]
struct User {
    id: UserId,
    name: String,
    gender: Gender,
}

// 帖子信息
#[derive(Debug)]
struct Topic {
    id:  TopicId,
    name: String,
    owner: UserId,
}

// 聊天事件 是一个标准的标签联合体，每种事件都有自己的数据结构
#[derive(Debug)]
enum ChatEvent {
    Join((UserId, TopicId)),
    Leave((UserId, TopicId)),
    Message((UserId, TopicId, String)),
}

// rust的匹配模式：可以用于struct或enum中匹配部分或全部内容
fn process_event(event: &ChatEvent) {
    match event {
        ChatEvent::Join((uid, _tid)) => println!("user {:?} joined", uid),
        ChatEvent::Leave((uid, tid)) => println!("user {:?} left {:?}", uid, tid),
        ChatEvent::Message((_, _, msg)) => println!("broadcast: {}", msg),
    }
}

fn main() {
    let zhangsan = User { id: UserId(1), name: "zhangsan".into(), gender: Gender::Female };
    let lisi = User { id: UserId(2), name: "lisi".into(), gender: Gender::Male };

    let topic = Topic { id: TopicId(1), name: "topic".into(), owner: UserId(1)};

    let event1 = ChatEvent::Join((zhangsan.id, topic.id));
    let event2 = ChatEvent::Join((lisi.id, topic.id));

    let event3 = ChatEvent::Message((zhangsan.id, topic.id, "hello-world".into()));

    let event4 = ChatEvent::Leave((zhangsan.id, topic.id));

    // 输出 event1: Join((UserId(1), TopicId(1))), event2: Join((UserId(2), TopicId(1))), event3: Message((UserId(1), TopicId(1), "hello-world")), event4: Leave((UserId(1), TopicId(1)))
    println!("event1: {:?}, event2: {:?}, event3: {:?}, event4: {:?}", event1, event2, event3, event4);

    println!("\nProcess event:");
    
    // 测试匹配
    process_event(&event1);
    process_event(&event2);
    process_event(&event3);
}
