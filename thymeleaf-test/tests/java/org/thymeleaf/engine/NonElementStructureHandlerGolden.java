package org.thymeleaf.engine;

import java.lang.reflect.Proxy;
import java.util.ArrayList;
import java.util.List;

import org.thymeleaf.context.IEngineContext;
import org.thymeleaf.context.ITemplateContext;
import org.thymeleaf.exceptions.TemplateProcessingException;
import org.thymeleaf.model.IModel;
import org.thymeleaf.model.IText;
import org.thymeleaf.processor.text.AbstractTextProcessor;
import org.thymeleaf.processor.text.ITextStructureHandler;
import org.thymeleaf.templatemode.TemplateMode;

/**
 * 固定 Thymeleaf 3.1.5 非元素 StructureHandler 的状态机与校验边界。
 */
public final class NonElementStructureHandlerGolden {

    private static final String BASELINE = "10f9dd2eb8cbd98515ce14b149d115e0287d0add";

    private NonElementStructureHandlerGolden() {
    }

    public static void main(final String[] args) {
        emit("baseline", BASELINE);
        text();
        cdata();
        comment();
        docType();
        processingInstruction();
        xmlDeclaration();
        templateBoundaries();
        abstractProcessorExceptions();
    }

    private static void text() {
        final TextStructureHandler handler = new TextStructureHandler();
        emit("text.new", textState(handler));

        final StringBuilder sequence = new StringBuilder("alpha");
        handler.setText(sequence);
        emit("text.sequence", textState(handler) + ",identity=" + (handler.setTextValue == sequence));

        handler.removeText();
        emitException("text.null", () -> handler.setText(null));
        emit("text.null.state", textState(handler));

        handler.removeText();
        emitException("text.model.null", () -> handler.replaceWith(null, true));
        emit("text.model.null.state", textState(handler));
    }

    private static void cdata() {
        final CDATASectionStructureHandler handler = new CDATASectionStructureHandler();
        handler.removeCDATASection();
        emitException("cdata.null", () -> handler.setContent(null));
        emit("cdata.null.state", handler.setContent + "," + handler.replaceWithModel + ","
                + handler.removeCDATASection);
        handler.removeCDATASection();
        emitException("cdata.model.null", () -> handler.replaceWith(null, false));
        emit("cdata.model.null.state", handler.setContent + "," + handler.replaceWithModel + ","
                + handler.removeCDATASection);
    }

    private static void comment() {
        final CommentStructureHandler handler = new CommentStructureHandler();
        handler.removeComment();
        emitException("comment.null", () -> handler.setContent(null));
        emit("comment.null.state", handler.setContent + "," + handler.replaceWithModel + ","
                + handler.removeComment);
        handler.removeComment();
        emitException("comment.model.null", () -> handler.replaceWith(null, true));
        emit("comment.model.null.state", handler.setContent + "," + handler.replaceWithModel + ","
                + handler.removeComment);
    }

    private static void docType() {
        final DocTypeStructureHandler handler = new DocTypeStructureHandler();
        handler.removeDocType();
        emitException("doctype.keyword.null",
                () -> handler.setDocType(null, null, "public", "system", "subset"));
        emit("doctype.keyword.null.state", docTypeState(handler));

        handler.removeDocType();
        emitException("doctype.element.null",
                () -> handler.setDocType("DOCTYPE", null, "public", "system", "subset"));
        emit("doctype.element.null.state", docTypeState(handler));

        handler.setDocType("DOCTYPE", "html", null, null, null);
        emit("doctype.optional.null", docTypeState(handler) + ",keyword=" + handler.setDocTypeKeyword
                + ",element=" + handler.setDocTypeElementName + ",public="
                + handler.setDocTypePublicId + ",system=" + handler.setDocTypeSystemId
                + ",subset=" + handler.setDocTypeInternalSubset);
    }

    private static void processingInstruction() {
        final ProcessingInstructionStructureHandler handler =
                new ProcessingInstructionStructureHandler();
        handler.removeProcessingInstruction();
        emitException("pi.target.null", () -> handler.setProcessingInstruction(null, null));
        emit("pi.target.null.state", processingInstructionState(handler));

        handler.removeProcessingInstruction();
        emitException("pi.content.null", () -> handler.setProcessingInstruction("xml", null));
        emit("pi.content.null.state", processingInstructionState(handler));

        handler.setProcessingInstruction("xml", "content");
        emit("pi.valid", processingInstructionState(handler) + ",target="
                + handler.setProcessingInstructionTarget + ",content="
                + handler.setProcessingInstructionContent);
    }

    private static void xmlDeclaration() {
        final XMLDeclarationStructureHandler handler = new XMLDeclarationStructureHandler();
        handler.removeXMLDeclaration();
        emitException("xml.keyword.null",
                () -> handler.setXMLDeclaration(null, "1.0", "UTF-8", "yes"));
        emit("xml.keyword.null.state", xmlDeclarationState(handler));

        handler.setXMLDeclaration("xml", null, null, null);
        emit("xml.optional.null", xmlDeclarationState(handler) + ",keyword="
                + handler.setXMLDeclarationKeyword + ",version="
                + handler.setXMLDeclarationVersion + ",encoding="
                + handler.setXMLDeclarationEncoding + ",standalone="
                + handler.setXMLDeclarationStandalone);
    }

    private static void templateBoundaries() {
        final TemplateBoundariesStructureHandler handler =
                new TemplateBoundariesStructureHandler();
        handler.setLocalVariable(null, null);
        handler.setLocalVariable(null, null);
        handler.removeLocalVariable(null);
        handler.removeLocalVariable(null);
        handler.setSelectionTarget(null);
        handler.setInliner(null);
        emit("boundary.null.context", boundaryState(handler));

        final List<String> contextCalls = new ArrayList<>();
        final IEngineContext engineContext = (IEngineContext) Proxy.newProxyInstance(
                NonElementStructureHandlerGolden.class.getClassLoader(),
                new Class<?>[] { IEngineContext.class },
                (proxy, method, arguments) -> {
                    if ("setVariables".equals(method.getName())) {
                        contextCalls.add("setVariables");
                    } else if ("removeVariable".equals(method.getName())) {
                        contextCalls.add("removeVariable:" + arguments[0]);
                    } else if ("setSelectionTarget".equals(method.getName())) {
                        contextCalls.add("setSelectionTarget:" + arguments[0]);
                    } else if ("setInliner".equals(method.getName())) {
                        contextCalls.add("setInliner:" + arguments[0]);
                    }
                    return defaultValue(method.getReturnType());
                });
        handler.applyContextModifications(engineContext);
        emit("boundary.apply.order", contextCalls);

        handler.insert("before", true);
        emit("boundary.text", boundaryState(handler) + ",text=" + handler.insertTextValue
                + ",processable=" + handler.insertTextProcessable);

        emitException("boundary.text.null", () -> handler.insert((String) null, false));
        emit("boundary.text.null.state", boundaryState(handler));

        final IModel model = (IModel) Proxy.newProxyInstance(
                NonElementStructureHandlerGolden.class.getClassLoader(),
                new Class<?>[] { IModel.class },
                (proxy, method, arguments) -> defaultValue(method.getReturnType()));
        handler.insert(model, false);
        emit("boundary.model", boundaryState(handler) + ",identity="
                + (handler.insertModelValue == model) + ",processable="
                + handler.insertModelProcessable);

        emitException("boundary.model.null", () -> handler.insert((IModel) null, true));
        emit("boundary.model.null.state", boundaryState(handler));

        handler.reset();
        emit("boundary.reset", boundaryState(handler));
    }

    private static void abstractProcessorExceptions() {
        final ThrowingTextProcessor processor = new ThrowingTextProcessor();

        final TemplateProcessingException noLocation =
                new TemplateProcessingException("plain");
        processor.failure = noLocation;
        final RuntimeException returnedNoLocation =
                capture(() -> processor.process(null, new Text("x"), new TextStructureHandler()));
        emit("processor.tpe.noLocation",
                processingExceptionState((TemplateProcessingException) returnedNoLocation)
                + ",identity=" + (returnedNoLocation == noLocation));

        final TemplateProcessingException enrich =
                new TemplateProcessingException("enrich");
        processor.failure = enrich;
        final RuntimeException returnedEnrich = capture(() -> processor.process(
                null, new Text("x", "page.html", 7, 11), new TextStructureHandler()));
        emit("processor.tpe.enrich",
                processingExceptionState((TemplateProcessingException) returnedEnrich)
                + ",identity=" + (returnedEnrich == enrich));

        final TemplateProcessingException preserve =
                new TemplateProcessingException("preserve", "own.html", 3, 4);
        processor.failure = preserve;
        final RuntimeException returnedPreserve = capture(() -> processor.process(
                null, new Text("x", "page.html", 7, 11), new TextStructureHandler()));
        emit("processor.tpe.preserve",
                processingExceptionState((TemplateProcessingException) returnedPreserve)
                + ",identity=" + (returnedPreserve == preserve));

        processor.failure = new IllegalStateException("boom");
        final TemplateProcessingException wrapped =
                (TemplateProcessingException) capture(() -> processor.process(
                        null, new Text("x", "page.html", 7, 11),
                        new TextStructureHandler()));
        emit("processor.wrap",
                processingExceptionState(wrapped) + ",cause="
                + wrapped.getCause().getClass().getName() + ":"
                + wrapped.getCause().getMessage());
    }

    private static String textState(final TextStructureHandler handler) {
        return handler.setText + "," + handler.replaceWithModel + "," + handler.removeText;
    }

    private static String docTypeState(final DocTypeStructureHandler handler) {
        return handler.setDocType + "," + handler.replaceWithModel + "," + handler.removeDocType;
    }

    private static String processingInstructionState(
            final ProcessingInstructionStructureHandler handler) {
        return handler.setProcessingInstruction + "," + handler.replaceWithModel + ","
                + handler.removeProcessingInstruction;
    }

    private static String xmlDeclarationState(final XMLDeclarationStructureHandler handler) {
        return handler.setXMLDeclaration + "," + handler.replaceWithModel + ","
                + handler.removeXMLDeclaration;
    }

    private static String boundaryState(final TemplateBoundariesStructureHandler handler) {
        return "text=" + handler.insertText
                + ",model=" + handler.insertModel
                + ",set=" + handler.setLocalVariable
                + ",setSize=" + (handler.addedLocalVariables == null
                        ? 0 : handler.addedLocalVariables.size())
                + ",setNull=" + (handler.addedLocalVariables != null
                        && handler.addedLocalVariables.containsKey(null))
                + ",remove=" + handler.removeLocalVariable
                + ",removeSize=" + (handler.removedLocalVariableNames == null
                        ? 0 : handler.removedLocalVariableNames.size())
                + ",removeNull=" + (handler.removedLocalVariableNames != null
                        && handler.removedLocalVariableNames.contains(null))
                + ",selection=" + handler.setSelectionTarget
                + ",selectionNull=" + (handler.selectionTargetObject == null)
                + ",inliner=" + handler.setInliner
                + ",inlinerNull=" + (handler.setInlinerValue == null);
    }

    private static Object defaultValue(final Class<?> type) {
        if (!type.isPrimitive()) {
            return null;
        }
        if (type == boolean.class) {
            return false;
        }
        if (type == char.class) {
            return '\0';
        }
        return 0;
    }

    private static String processingExceptionState(
            final TemplateProcessingException exception) {
        return "message=" + exception.getMessage()
                + ",template=" + exception.getTemplateName()
                + ",line=" + exception.getLine()
                + ",col=" + exception.getCol();
    }

    private static RuntimeException capture(final ThrowingRunnable operation) {
        try {
            operation.run();
            throw new AssertionError("operation should fail");
        } catch (final RuntimeException exception) {
            return exception;
        }
    }

    private static void emitException(final String key, final ThrowingRunnable operation) {
        try {
            operation.run();
            emit(key, "NONE");
        } catch (final RuntimeException exception) {
            emit(key, exception.getClass().getName() + ":" + exception.getMessage());
        }
    }

    private static void emit(final String key, final Object value) {
        System.out.println(key + "=" + value);
    }

    @FunctionalInterface
    private interface ThrowingRunnable {
        void run();
    }

    private static final class ThrowingTextProcessor extends AbstractTextProcessor {

        private RuntimeException failure;

        private ThrowingTextProcessor() {
            super(TemplateMode.HTML, 100);
        }

        @Override
        protected void doProcess(
                final ITemplateContext context,
                final IText text,
                final ITextStructureHandler structureHandler) {
            throw this.failure;
        }
    }
}
