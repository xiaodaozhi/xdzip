#include "RuntimeConnection.h"
#include <QElapsedTimer>
#include <QPointer>
#include <atomic>
#include <condition_variable>
#include <deque>
#include <mutex>

namespace {
constexpr std::size_t MailboxCapacity = 256;
constexpr std::size_t MailboxBytes = 8 * 1024 * 1024;
std::atomic<int> liveReceivers{0};
bool isPayload(const xdzip::reader::Event &event) {
    using xdzip::reader::EventKind;
    return event.kind == EventKind::Page || event.kind == EventKind::Preflight ||
           event.kind == EventKind::Snapshot || event.kind == EventKind::TaskUpdate ||
           event.kind == EventKind::TaskAccepted || event.kind == EventKind::TargetChecked;
}
std::size_t payloadBytes(const xdzip::reader::Event &event) {
    std::size_t bytes = event.error.size() + event.report.declared_bytes.size();
    for (const auto &row : event.rows)
        bytes += row.name.size() + row.id.size() + row.navigation_id.size();
    for (const auto &node : event.directories)
        bytes += node.id.size() + node.parent.size() + node.name.size() + node.path.size();
    for (const auto &finding : event.report.findings)
        bytes += finding.size();
    bytes += event.task.status.size() + event.task.target.size() + event.task.current.size() +
             event.task.error.size() + event.task.prompt_name.size();
    return bytes;
}
} // namespace
struct RuntimeMailbox {
    std::mutex mutex;
    std::condition_variable space;
    std::deque<xdzip::reader::Event> queue;
    std::size_t bytes = 0;
    std::size_t payloads = 0;
    bool notified = false;
    bool closed = false;
    std::size_t peakQueued = 0;
    std::size_t peakBytes = 0;
    std::size_t notifications = 0;
};
RuntimeReceiver::RuntimeReceiver(rust::Box<xdzip::reader::EventReceiver> receiver,
                                 std::shared_ptr<RuntimeMailbox> mailbox)
    : m_receiver(std::move(receiver)), m_mailbox(std::move(mailbox)) {
    ++liveReceivers;
    setObjectName("xdzip-event-receiver");
}
RuntimeReceiver::~RuntimeReceiver() {
    --liveReceivers;
}
int RuntimeReceiver::activeReceivers() {
    return liveReceivers.load();
}
void RuntimeReceiver::run() {
    try {
        for (;;) {
            auto event = m_receiver->next_event();
            if (event.kind == xdzip::reader::EventKind::End)
                break;
            const auto bytes = payloadBytes(event);
            if (bytes > MailboxBytes)
                break; // connection loss is reported, never silently dropped
            bool notify = false;
            {
                std::unique_lock<std::mutex> lock(m_mailbox->mutex);
                m_mailbox->space.wait(lock, [&] {
                    return m_mailbox->closed ||
                           (m_mailbox->queue.size() < MailboxCapacity &&
                            (!isPayload(event) || m_mailbox->payloads < MailboxCapacity / 2) &&
                            m_mailbox->bytes + bytes <= MailboxBytes);
                });
                if (m_mailbox->closed)
                    break;
                m_mailbox->bytes += bytes;
                if (isPayload(event))
                    ++m_mailbox->payloads;
                m_mailbox->queue.push_back(std::move(event));
                m_mailbox->peakQueued = std::max(m_mailbox->peakQueued, m_mailbox->queue.size());
                m_mailbox->peakBytes = std::max(m_mailbox->peakBytes, m_mailbox->bytes);
                if (!m_mailbox->notified) {
                    m_mailbox->notified = true;
                    notify = true;
                    ++m_mailbox->notifications;
                }
            }
            if (notify)
                emit available();
        }
    } catch (const std::exception &) {
        // Pending requests are failed by disconnected(false), without crossing an exception into
        // Qt.
    }
}
RuntimeConnection::RuntimeConnection(QObject *parent)
    : QObject(parent), m_mailbox(std::make_shared<RuntimeMailbox>()) {
    try {
        auto runtime = xdzip::reader::new_runtime();
        m_client.emplace(runtime->take_client());
        auto *receiver = new RuntimeReceiver(runtime->take_receiver(), m_mailbox);
        connect(receiver, &RuntimeReceiver::available, this, &RuntimeConnection::drain,
                Qt::QueuedConnection);
        connect(receiver, &QThread::finished, receiver, &QObject::deleteLater);
        connect(
            receiver, &QThread::finished, this,
            [this] {
                m_receiverFinished = true;
                bool schedule = false;
                {
                    std::lock_guard<std::mutex> lock(m_mailbox->mutex);
                    if (!m_mailbox->notified) {
                        m_mailbox->notified = true;
                        schedule = true;
                    }
                }
                if (schedule)
                    QMetaObject::invokeMethod(this, &RuntimeConnection::drain,
                                              Qt::QueuedConnection);
            },
            Qt::QueuedConnection);
        receiver->start();
    } catch (const std::exception &) {
        m_stopped = true;
        QMetaObject::invokeMethod(this, [this] { emit disconnected(false); }, Qt::QueuedConnection);
    }
}
RuntimeConnection::~RuntimeConnection() {
    shutdown();
    {
        std::lock_guard<std::mutex> lock(m_mailbox->mutex);
        m_mailbox->closed = true;
    }
    m_mailbox->space.notify_all();
    // Qt removes receiver notifications targeting this object. The QThread deletes itself on
    // finish.
}
xdzip::reader::Submission RuntimeConnection::submit(xdzip::reader::Command command) {
    if (!m_client || m_stopped)
        return {xdzip::reader::Admission::Closed, 0};
    return (*m_client)->submit(std::move(command));
}
void RuntimeConnection::cancel(quint64 request) {
    if (m_client && !m_stopped)
        (*m_client)->cancel(request);
}
bool RuntimeConnection::cancelTask(quint64 task) {
    return m_client && !m_stopped && (*m_client)->cancel_task(task);
}
bool RuntimeConnection::decideTask(quint64 task, quint64 prompt, quint8 policy, bool all) {
    return m_client && !m_stopped && (*m_client)->decide_task(task, prompt, policy, all);
}
void RuntimeConnection::shutdown() {
    if (m_client)
        (*m_client)->shutdown();
}
void RuntimeConnection::drain() {
    QElapsedTimer budget;
    budget.start();
    for (int count = 0; count < 32 && budget.elapsed() < 4; ++count) {
        std::optional<xdzip::reader::Event> event;
        {
            std::lock_guard<std::mutex> lock(m_mailbox->mutex);
            if (m_mailbox->queue.empty())
                break;
            m_mailbox->bytes -= payloadBytes(m_mailbox->queue.front());
            if (isPayload(m_mailbox->queue.front()))
                --m_mailbox->payloads;
            event.emplace(std::move(m_mailbox->queue.front()));
            m_mailbox->queue.pop_front();
        }
        m_mailbox->space.notify_one();
        if (event->sequence <= m_sequence)
            continue;
        m_sequence = event->sequence;
        if (event->kind == xdzip::reader::EventKind::ShutdownReady)
            m_shutdownSeen = true;
        QPointer<RuntimeConnection> alive(this);
        const auto consumer = m_consumer;
        if (consumer)
            consumer(*event);
        if (!alive)
            return;
    }
    bool more;
    {
        std::lock_guard<std::mutex> lock(m_mailbox->mutex);
        more = !m_mailbox->queue.empty();
        if (!more)
            m_mailbox->notified = false;
    }
    // Exactly one notification chain, even if the mailbox is full. No per-event Qt callbacks.
    if (more)
        QMetaObject::invokeMethod(this, &RuntimeConnection::drain, Qt::QueuedConnection);
    else if (m_receiverFinished && !m_stopped) {
        m_stopped = true;
        emit disconnected(m_shutdownSeen);
    }
}

RuntimeConnection::Statistics RuntimeConnection::statistics() const {
    std::lock_guard<std::mutex> lock(m_mailbox->mutex);
    return {m_mailbox->peakQueued, m_mailbox->peakBytes, m_mailbox->notifications};
}
